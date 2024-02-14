use std::{
    borrow::Cow,
    collections::{vec_deque, VecDeque},
    fs::{File, OpenOptions},
    io::SeekFrom,
    ops::Index,
    path::{Path, PathBuf},
    time::SystemTime,
};

use fd_lock::RwLock;
use rustyline::{
    history::{History, SearchDirection, SearchResult},
    Config, HistoryDuplicates, Result,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub entry: String,
    pub working_dir: String,
}

#[derive(Default)]
pub struct JsonHistory {
    mem: MemHistory,
    /// Number of entries inputted by user and not saved yet
    new_entries: usize,
    /// last path used by either `load` or `save`
    path_info: Option<PathInfo>,
}

/// Last histo path, modified timestamp and size
#[derive(Clone)]
struct PathInfo(PathBuf, SystemTime, usize);

impl JsonHistory {
    /// Default constructor
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(Config::default())
    }

    /// Customized constructor with:
    /// - `Config::max_history_size()`,
    /// - `Config::history_ignore_space()`,
    /// - `Config::history_duplicates()`.
    #[must_use]
    pub fn with_config(config: Config) -> Self {
        Self {
            mem: MemHistory::with_config(config),
            new_entries: 0,
            path_info: None,
        }
    }

    pub fn get_hint(&self, term: &str) -> Option<SearchResult> {
        if term.is_empty() {
            return None;
        }
        let cwd = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| String::new());
        let mut first_wrong_dir = None;
        for (i, entry) in self.mem.entries.iter().enumerate() {
            if entry.entry.starts_with(term) {
                let line = entry.entry.clone();
                if entry.working_dir == cwd {
                    return Some(SearchResult {
                        pos: term.len(),
                        idx: i,
                        entry: line.into(),
                    });
                } else {
                    first_wrong_dir = Some(SearchResult {
                        pos: term.len(),
                        idx: i,
                        entry: line.into(),
                    });
                }
            }
        }
        first_wrong_dir
    }

    fn save_to(&mut self, file: &File, append: bool) -> Result<()> {
        use std::io::{BufWriter, Write};

        fix_perm(file);
        let mut wtr = BufWriter::new(file);
        let first_new_entry = if append {
            self.mem.len().saturating_sub(self.new_entries)
        } else {
            0
        };
        for entry in self.mem.entries.iter().skip(first_new_entry) {
            let s = serde_json::to_string(entry).unwrap();
            wtr.write_all(s.as_bytes())?;
            wtr.write_all(b"\n")?;
        }
        // https://github.com/rust-lang/rust/issues/32677#issuecomment-204833485
        wtr.flush()?;
        Ok(())
    }

    fn load_from(&mut self, file: &File) -> Result<()> {
        use std::io::{BufRead, BufReader};

        let rdr = BufReader::new(file);
        for line in rdr.lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }

            let entry: HistoryEntry = serde_json::from_str(&line)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, Box::new(e)))?;
            self.mem.add_entry(entry);
        }
        self.new_entries = 0; // TODO we may lost new entries if loaded lines < max_len
        Ok(())
    }

    fn update_path(&mut self, path: &Path, file: &File, size: usize) -> Result<()> {
        let modified = file.metadata()?.modified()?;
        if let Some(PathInfo(
            ref mut previous_path,
            ref mut previous_modified,
            ref mut previous_size,
        )) = self.path_info
        {
            if previous_path.as_path() != path {
                *previous_path = path.to_owned();
            }
            *previous_modified = modified;
            *previous_size = size;
        } else {
            self.path_info = Some(PathInfo(path.to_owned(), modified, size));
        }
        Ok(())
    }

    fn can_just_append(&self, path: &Path, file: &File) -> Result<bool> {
        if let Some(PathInfo(ref previous_path, ref previous_modified, ref previous_size)) =
            self.path_info
        {
            if previous_path.as_path() != path {
                return Ok(false);
            }
            let modified = file.metadata()?.modified()?;
            if *previous_modified != modified
                || self.mem.max_len <= *previous_size
                || self.mem.max_len < (*previous_size).saturating_add(self.new_entries)
            {
                Ok(false)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    /// Return a forward iterator.
    #[must_use]
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &HistoryEntry> + '_ {
        self.mem.entries.iter()
    }
}

impl History for JsonHistory {
    fn get(&self, index: usize, dir: SearchDirection) -> Result<Option<SearchResult>> {
        self.mem.get(index, dir)
    }

    fn add(&mut self, line: &str) -> Result<bool> {
        if self.mem.add(line)? {
            self.new_entries = self.new_entries.saturating_add(1).min(self.len());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn add_owned(&mut self, line: String) -> Result<bool> {
        if self.mem.add_owned(line)? {
            self.new_entries = self.new_entries.saturating_add(1).min(self.len());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn len(&self) -> usize {
        self.mem.len()
    }

    fn is_empty(&self) -> bool {
        self.mem.is_empty()
    }

    fn set_max_len(&mut self, len: usize) -> Result<()> {
        self.mem.set_max_len(len)?;
        self.new_entries = self.new_entries.min(len);
        Ok(())
    }

    fn ignore_dups(&mut self, yes: bool) -> Result<()> {
        self.mem.ignore_dups(yes)
    }

    fn ignore_space(&mut self, yes: bool) {
        self.mem.ignore_space(yes);
    }

    fn save(&mut self, path: &Path) -> Result<()> {
        if self.is_empty() || self.new_entries == 0 {
            return Ok(());
        }
        let old_umask = umask();
        let f = File::create(path);
        restore_umask(old_umask);
        let file = f?;
        let mut lock = RwLock::new(file);
        let lock_guard = lock.write()?;
        self.save_to(&lock_guard, false)?;
        self.new_entries = 0;
        self.update_path(path, &lock_guard, self.len())
    }

    fn append(&mut self, path: &Path) -> Result<()> {
        use std::io::Seek;

        if self.is_empty() || self.new_entries == 0 {
            return Ok(());
        }
        if !path.exists() || self.new_entries == self.mem.max_len {
            return self.save(path);
        }
        let file = OpenOptions::new().write(true).read(true).open(path)?;
        let mut lock = RwLock::new(file);
        let mut lock_guard = lock.write()?;
        if self.can_just_append(path, &lock_guard)? {
            lock_guard.seek(SeekFrom::End(0))?;
            self.save_to(&lock_guard, true)?;
            let size = self
                .path_info
                .as_ref()
                .unwrap()
                .2
                .saturating_add(self.new_entries);
            self.new_entries = 0;
            return self.update_path(path, &lock_guard, size);
        }
        // we may need to truncate file before appending new entries
        let mut other = Self {
            mem: MemHistory {
                entries: VecDeque::new(),
                max_len: self.mem.max_len,
                ignore_space: self.mem.ignore_space,
                ignore_dups: self.mem.ignore_dups,
            },
            new_entries: 0,
            path_info: None,
        };
        other.load_from(&lock_guard)?;
        let first_new_entry = self.mem.len().saturating_sub(self.new_entries);
        for entry in self.mem.entries.iter().skip(first_new_entry) {
            other.mem.add_entry(entry.clone());
        }
        lock_guard.seek(SeekFrom::Start(0))?;
        lock_guard.set_len(0)?; // if new size < old size
        other.save_to(&lock_guard, false)?;
        self.update_path(path, &lock_guard, other.len())?;
        self.new_entries = 0;
        Ok(())
    }

    fn load(&mut self, path: &Path) -> Result<()> {
        let file = File::open(path)?;
        let lock = RwLock::new(file);
        let lock_guard = lock.read()?;
        let len = self.len();
        self.load_from(&lock_guard)?;
        self.update_path(path, &lock_guard, self.len() - len)
    }

    fn clear(&mut self) -> Result<()> {
        self.mem.clear()?;
        self.new_entries = 0;
        if let Some(PathInfo(path, _, _)) = self.path_info.clone() {
            let file = OpenOptions::new().write(true).read(true).open(&path)?;
            let mut lock = RwLock::new(file);
            let lock_guard = lock.write()?;
            lock_guard.set_len(0)?;
            self.update_path(&path, &lock_guard, 0)?;
        }

        Ok(())
    }

    fn search(
        &self,
        term: &str,
        start: usize,
        dir: SearchDirection,
    ) -> Result<Option<SearchResult>> {
        self.mem.search(term, start, dir)
    }

    fn starts_with(
        &self,
        term: &str,
        start: usize,
        dir: SearchDirection,
    ) -> Result<Option<SearchResult>> {
        self.mem.starts_with(term, start, dir)
    }
}

impl Index<usize> for JsonHistory {
    type Output = HistoryEntry;

    fn index(&self, index: usize) -> &HistoryEntry {
        &self.mem.entries[index]
    }
}

impl<'a> IntoIterator for &'a JsonHistory {
    type IntoIter = vec_deque::Iter<'a, HistoryEntry>;
    type Item = &'a HistoryEntry;

    fn into_iter(self) -> Self::IntoIter {
        self.mem.entries.iter()
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(windows, target_arch = "wasm32"))] {
        fn umask() -> u16 {
            0
        }

        fn restore_umask(_: u16) {}

        fn fix_perm(_: &File) {}
    } else if #[cfg(unix)] {
        use nix::sys::stat::{self, Mode, fchmod};
        fn umask() -> Mode {
            stat::umask(Mode::S_IXUSR | Mode::S_IRWXG | Mode::S_IRWXO)
        }

        fn restore_umask(old_umask: Mode) {
            stat::umask(old_umask);
        }

        fn fix_perm(file: &File) {
            use std::os::unix::io::AsRawFd;
            let _ = fchmod(file.as_raw_fd(), Mode::S_IRUSR | Mode::S_IWUSR);
        }
    }
}

struct MemHistory {
    entries: VecDeque<HistoryEntry>,
    max_len: usize,
    ignore_space: bool,
    ignore_dups: bool,
}

impl MemHistory {
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(Config::default())
    }

    #[must_use]
    pub fn with_config(config: Config) -> Self {
        Self {
            entries: VecDeque::new(),
            max_len: config.max_history_size(),
            ignore_space: config.history_ignore_space(),
            ignore_dups: config.history_duplicates() == HistoryDuplicates::IgnoreConsecutive,
        }
    }

    fn search_match<F>(
        &self,
        term: &str,
        start: usize,
        dir: SearchDirection,
        test: F,
    ) -> Option<SearchResult>
    where
        F: Fn(&str) -> Option<usize>,
    {
        if term.is_empty() || start >= self.len() {
            return None;
        }
        match dir {
            SearchDirection::Reverse => {
                for (idx, entry) in self
                    .entries
                    .iter()
                    .rev()
                    .skip(self.len() - 1 - start)
                    .enumerate()
                {
                    if let Some(cursor) = test(&entry.entry) {
                        return Some(SearchResult {
                            idx: start - idx,
                            entry: Cow::Borrowed(&entry.entry),
                            pos: cursor,
                        });
                    }
                }
                None
            }
            SearchDirection::Forward => {
                for (idx, entry) in self.entries.iter().skip(start).enumerate() {
                    if let Some(cursor) = test(&entry.entry) {
                        return Some(SearchResult {
                            idx: idx + start,
                            entry: Cow::Borrowed(&entry.entry),
                            pos: cursor,
                        });
                    }
                }
                None
            }
        }
    }

    fn ignore(&self, line: &str) -> bool {
        if self.max_len == 0 {
            return true;
        }
        if line.is_empty()
            || (self.ignore_space && line.chars().next().map_or(true, char::is_whitespace))
        {
            return true;
        }
        if self.ignore_dups {
            if let Some(s) = self.entries.back() {
                if s.entry == line {
                    return true;
                }
            }
        }
        false
    }

    fn insert(&mut self, line: String) {
        if self.entries.len() == self.max_len {
            self.entries.pop_front();
        }

        let entry = HistoryEntry {
            entry: line,
            working_dir: std::env::current_dir()
                .map(|dir| dir.to_string_lossy().to_string())
                .unwrap_or_else(|_| String::new()),
        };

        self.entries.push_back(entry);
    }

    fn add_entry(&mut self, entry: HistoryEntry) {
        if self.entries.len() == self.max_len {
            self.entries.pop_front();
        }

        self.entries.push_back(entry);
    }
}

impl Default for MemHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl History for MemHistory {
    fn get(&self, index: usize, _: SearchDirection) -> Result<Option<SearchResult>> {
        Ok(self
            .entries
            .get(index)
            .map(|e| e.entry.as_str())
            .map(Cow::Borrowed)
            .map(|entry| SearchResult {
                entry,
                idx: index,
                pos: 0,
            }))
    }

    fn add(&mut self, line: &str) -> Result<bool> {
        if self.ignore(line) {
            return Ok(false);
        }
        self.insert(line.to_owned());
        Ok(true)
    }

    fn add_owned(&mut self, line: String) -> Result<bool> {
        if self.ignore(&line) {
            return Ok(false);
        }
        self.insert(line);
        Ok(true)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn set_max_len(&mut self, len: usize) -> Result<()> {
        self.max_len = len;
        if self.len() > len {
            self.entries.drain(..self.len() - len);
        }
        Ok(())
    }

    fn ignore_dups(&mut self, yes: bool) -> Result<()> {
        self.ignore_dups = yes;
        Ok(())
    }

    fn ignore_space(&mut self, yes: bool) {
        self.ignore_space = yes;
    }

    fn save(&mut self, _: &Path) -> Result<()> {
        unimplemented!();
    }

    fn append(&mut self, _: &Path) -> Result<()> {
        unimplemented!();
    }

    fn load(&mut self, _: &Path) -> Result<()> {
        unimplemented!();
    }

    fn clear(&mut self) -> Result<()> {
        self.entries.clear();
        Ok(())
    }

    fn search(
        &self,
        term: &str,
        start: usize,
        dir: SearchDirection,
    ) -> Result<Option<SearchResult>> {
        use regex::{escape, RegexBuilder};
        Ok(
            if let Ok(re) = RegexBuilder::new(&escape(term))
                .case_insensitive(true)
                .build()
            {
                let test = |entry: &str| re.find(entry).map(|m| m.start());
                self.search_match(term, start, dir, test)
            } else {
                None
            },
        )
    }

    fn starts_with(
        &self,
        term: &str,
        start: usize,
        dir: SearchDirection,
    ) -> Result<Option<SearchResult>> {
        use regex::{escape, RegexBuilder};
        Ok(
            if let Ok(re) = RegexBuilder::new(&escape(term))
                .case_insensitive(true)
                .build()
            {
                let test = |entry: &str| {
                    re.find(entry)
                        .and_then(|m| if m.start() == 0 { Some(m) } else { None })
                        .map(|m| m.end())
                };
                self.search_match(term, start, dir, test)
            } else {
                None
            },
        )
    }
}

impl Index<usize> for MemHistory {
    type Output = String;

    fn index(&self, index: usize) -> &String {
        &self.entries[index].entry
    }
}

impl<'a> IntoIterator for &'a MemHistory {
    type IntoIter = vec_deque::Iter<'a, HistoryEntry>;
    type Item = &'a HistoryEntry;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}
