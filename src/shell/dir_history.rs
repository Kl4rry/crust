use std::path::{Path, PathBuf};

use crate::parser::shell_error::ShellErrorKind;

pub struct DirHistory {
    dir_history: Vec<PathBuf>,
}

impl DirHistory {
    pub fn new() -> Self {
        Self {
            dir_history: Vec::new(),
        }
    }

    pub fn change_dir(&mut self, dir: impl AsRef<Path>) -> Result<(), ShellErrorKind> {
        let old_dir = std::env::current_dir().map_err(|err| ShellErrorKind::Io(None, err))?;
        self.dir_history.push(old_dir);

        std::env::set_current_dir(dir).map_err(|err| ShellErrorKind::Io(None, err))
    }

    pub fn back(&mut self) -> Result<(), ShellErrorKind> {
        if let Some(path) = self.dir_history.pop() {
            std::env::set_current_dir(path).map_err(|err| ShellErrorKind::Io(None, err))?;
        }
        Ok(())
    }
}

impl Default for DirHistory {
    fn default() -> Self {
        Self::new()
    }
}
