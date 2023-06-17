use std::path::{Path, PathBuf};

use crate::parser::shell_error::ShellErrorKind;

pub struct DirHistory {
    dir_history: Vec<PathBuf>,
    dir_history_index: i64,
}

impl DirHistory {
    pub fn new() -> Self {
        Self {
            dir_history: Vec::new(),
            dir_history_index: -1,
        }
    }

    pub fn change_dir(&mut self, dir: impl AsRef<Path>) -> Result<(), ShellErrorKind> {
        let old_dir = std::env::current_dir().map_err(|err| ShellErrorKind::Io(None, err))?;
        self.push_dir(old_dir);

        std::env::set_current_dir(dir).map_err(|err| ShellErrorKind::Io(None, err))
    }

    fn push_dir(&mut self, dir: PathBuf) {
        if self.dir_history_index + 1 < self.dir_history.len() as i64 {
            for _ in 0..(self.dir_history.len() as i64 - self.dir_history_index + 1) {
                self.dir_history.pop();
            }
        }
        self.dir_history.push(dir);
        self.dir_history_index += 1;
    }

    pub fn back(&mut self) -> Result<(), ShellErrorKind> {
        if self.dir_history_index < 0 {
            return Ok(());
        }

        let path = &self.dir_history[self.dir_history_index as usize];
        self.dir_history_index -= 1;
        std::env::set_current_dir(path).map_err(|err| ShellErrorKind::Io(None, err))
    }
}

impl Default for DirHistory {
    fn default() -> Self {
        Self::new()
    }
}
