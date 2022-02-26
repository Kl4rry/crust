// This file contains a modded version of the rustyline file completer
use std::{
    borrow::Cow,
    cmp,
    env::current_dir,
    fs,
    path::Path,
};

use directories::BaseDirs;
use memchr::memchr;
use rustyline::{
    completion::{escape, extract_word, unescape, Candidate, Completer, Pair, Quote},
    Context, Result,
};

use crate::parser::lexer::escape_char;

pub struct FilenameCompleter {
    break_chars: &'static [u8],
    double_quotes_special_chars: &'static [u8],
}

const DOUBLE_QUOTES_ESCAPE_CHAR: Option<char> = Some('\\');

// rl_basic_word_break_characters, rl_completer_word_break_characters
const DEFAULT_BREAK_CHARS: [u8; 19] = [
    b' ', b'\t', b'\n', b'"', b'\\', b'\'', b'@', b'$', b'>', b'<', b'=', b';', b'|', b'&',
    b'{', b'(', b'\0', b'}', b')',
];
const ESCAPE_CHAR: Option<char> = Some('\\');
// In double quotes, not all break_chars need to be escaped
// https://www.gnu.org/software/bash/manual/html_node/Double-Quotes.html
const DOUBLE_QUOTES_SPECIAL_CHARS: [u8; 3] = [b'"', b'$', b'\\'];

fn replace_escapes(line: &str, pos: usize) -> (String, usize) {
    if line.is_empty() {
        return (String::new(), 0);
    }

    let mut index = 0;
    let mut string = String::new();
    let mut new_pos = pos;
    while let Some(new_index) = memchr(b'\\', &line.as_bytes()[index..]) {
        if new_index < new_pos {
            new_pos -= 1;
        }
        unsafe {
            string
                .as_mut_vec()
                .extend_from_slice(line[index..new_index].as_bytes())
        };

        let escape = escape_char(*line.as_bytes().get(new_index + 1).unwrap_or(&b'\\'));
        unsafe { string.as_mut_vec().push(escape) }
        index = cmp::min(new_index + 2, line.len() - 1);
    }
    unsafe {
        string
            .as_mut_vec()
            .extend_from_slice(&line.as_bytes()[index..])
    }
    (string, new_pos)
}

impl FilenameCompleter {
    pub fn new() -> Self {
        Self {
            break_chars: &DEFAULT_BREAK_CHARS,
            double_quotes_special_chars: &DOUBLE_QUOTES_SPECIAL_CHARS,
        }
    }

    pub fn complete_path(&self, line: &str, pos: usize) -> Result<(usize, Vec<Pair>)> {
        let (line, pos) = replace_escapes(line, pos);
        let line = &*line;
        let (start, path, esc_char, break_chars, quote) =
            if let Some((idx, quote)) = find_unclosed_quote(&line[..pos]) {
                let start = idx + 1;
                if quote == Quote::Double {
                    (
                        start,
                        unescape(&line[start..pos], DOUBLE_QUOTES_ESCAPE_CHAR),
                        DOUBLE_QUOTES_ESCAPE_CHAR,
                        &self.double_quotes_special_chars,
                        quote,
                    )
                } else {
                    (
                        start,
                        Cow::Borrowed(&line[start..pos]),
                        None,
                        &self.break_chars,
                        quote,
                    )
                }
            } else {
                let (start, path) = extract_word(line, pos, ESCAPE_CHAR, self.break_chars);
                let path = unescape(path, ESCAPE_CHAR);
                (start, path, ESCAPE_CHAR, &self.break_chars, Quote::None)
            };
        let mut matches = filename_complete(&path, esc_char, break_chars, quote);
        #[allow(clippy::unnecessary_sort_by)]
        matches.sort_by(|a, b| a.display().cmp(b.display()));
        Ok((start, matches))
    }
}

impl Default for FilenameCompleter {
    fn default() -> Self {
        Self::new()
    }
}

impl Completer for FilenameCompleter {
    type Candidate = Pair;

    fn complete(&self, line: &str, pos: usize, _ctx: &Context<'_>) -> Result<(usize, Vec<Pair>)> {
        self.complete_path(line, pos)
    }
}

fn filename_complete(
    path: &str,
    esc_char: Option<char>,
    break_chars: &[u8],
    quote: Quote,
) -> Vec<Pair> {
    let sep = '/';
    let (dir_name, file_name) = match path.rfind(sep) {
        Some(idx) => path.split_at(idx + sep.len_utf8()),
        None => ("", path),
    };

    let dir_path = Path::new(dir_name);
    let dir = if dir_path.starts_with("~") {
        {
            if let Some(base_dirs) = BaseDirs::new() {
                let home = base_dirs.home_dir();
                match dir_path.strip_prefix("~") {
                    Ok(rel_path) => home.join(rel_path),
                    _ => home.to_path_buf(),
                }
            } else {
                dir_path.to_path_buf()
            }
        }
    } else if dir_path.is_relative() {
        // TODO ~user[/...] (https://crates.io/crates/users)
        if let Ok(cwd) = current_dir() {
            cwd.join(dir_path)
        } else {
            dir_path.to_path_buf()
        }
    } else {
        dir_path.to_path_buf()
    };

    let mut entries: Vec<Pair> = Vec::new();

    // if dir doesn't exist, then don't offer any completions
    if !dir.exists() {
        return entries;
    }

    // if any of the below IO operations have errors, just ignore them
    if let Ok(read_dir) = dir.read_dir() {
        let file_name = normalize(file_name);
        for entry in read_dir.flatten() {
            if let Some(s) = entry.file_name().to_str() {
                let ns = normalize(s);
                if ns.starts_with(file_name.as_ref()) {
                    if let Ok(metadata) = fs::metadata(entry.path()) {
                        let mut path = String::from(dir_name) + s;
                        if metadata.is_dir() {
                            path.push(sep);
                        }
                        path = path.replace('\\', "/");
                        entries.push(Pair {
                            display: String::from(s),
                            replacement: escape(path, esc_char, break_chars, quote),
                        });
                    } // else ignore PermissionDenied
                }
            }
        }
    }
    entries
}

#[cfg(any(windows, target_os = "macos"))]
fn normalize(s: &str) -> Cow<str> {
    // case insensitive
    Cow::Owned(s.to_lowercase())
}

#[cfg(not(any(windows, target_os = "macos")))]
fn normalize(s: &str) -> Cow<str> {
    Cow::Borrowed(s)
}

#[derive(PartialEq)]
enum ScanMode {
    DoubleQuote,
    Escape,
    EscapeInDoubleQuote,
    Normal,
    SingleQuote,
}

fn find_unclosed_quote(s: &str) -> Option<(usize, Quote)> {
    let char_indices = s.char_indices();
    let mut mode = ScanMode::Normal;
    let mut quote_index = 0;
    for (index, char) in char_indices {
        match mode {
            ScanMode::DoubleQuote => {
                if char == '"' {
                    mode = ScanMode::Normal;
                } else if char == '\\' {
                    // both windows and unix support escape in double quote
                    mode = ScanMode::EscapeInDoubleQuote;
                }
            }
            ScanMode::Escape => {
                mode = ScanMode::Normal;
            }
            ScanMode::EscapeInDoubleQuote => {
                mode = ScanMode::DoubleQuote;
            }
            ScanMode::Normal => {
                if char == '"' {
                    mode = ScanMode::DoubleQuote;
                    quote_index = index;
                } else if char == '\\' {
                    mode = ScanMode::Escape;
                } else if char == '\'' {
                    mode = ScanMode::SingleQuote;
                    quote_index = index;
                }
            }
            ScanMode::SingleQuote => {
                if char == '\'' {
                    mode = ScanMode::Normal;
                } // no escape in single quotes
            }
        };
    }
    if ScanMode::DoubleQuote == mode || ScanMode::EscapeInDoubleQuote == mode {
        return Some((quote_index, Quote::Double));
    } else if ScanMode::SingleQuote == mode {
        return Some((quote_index, Quote::Single));
    }
    None
}
