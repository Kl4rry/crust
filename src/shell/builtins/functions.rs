use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use phf::*;

use crate::{
    parser::shell_error::ShellErrorKind,
    shell::{
        frame::Frame,
        stream::{OutputStream, ValueStream},
        value::SpannedValue,
        Shell,
    },
};

mod alias;
mod assert;
mod cd;
mod clear;
mod echo;
mod env;
mod exit;
mod help;
mod import;
mod load;
mod open;
mod pwd;
mod save;
mod unalias;

pub type BulitinFn = fn(
    &mut Shell,
    &mut Frame,
    Vec<SpannedValue>,
    ValueStream,
    &mut OutputStream,
) -> Result<(), ShellErrorKind>;

static BUILTIN_FUNCTIONS: phf::Map<&'static str, BulitinFn> = phf_map! {
    "clear" => clear::clear,
    "pwd" => pwd::pwd,
    "exit" => exit::exit,
    "cd" => cd::cd,
    "echo" => echo::echo,
    "import" => import::import,
    "alias" => alias::alias,
    "unalias" => unalias::unalias,
    "env" => env::env,
    "open" => open::open,
    "load" => load::load,
    "save" => save::save,
    "help" => help::help,
    "assert" => assert::assert,
};

pub fn get_builtin(command: &str) -> Option<BulitinFn> {
    BUILTIN_FUNCTIONS.get(command).copied()
}

pub fn get_builtins() -> Vec<&'static str> {
    BUILTIN_FUNCTIONS.keys().copied().collect()
}

pub fn read_file(path: impl AsRef<Path>) -> Result<String, ShellErrorKind> {
    let path = path.as_ref();
    fs::read_to_string(path)
        .map_err(|e| file_err_to_shell_err(e, path.to_string_lossy().to_string()))
}

pub fn save_file(path: impl AsRef<Path>, data: &[u8], append: bool) -> Result<(), ShellErrorKind> {
    let path = path.as_ref();
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .append(append)
        .open(path)
        .map_err(|e| file_err_to_shell_err(e, path.to_string_lossy().to_string()))?;
    file.write_all(data)
        .map_err(|e| file_err_to_shell_err(e, path.to_string_lossy().to_string()))
}

pub fn file_err_to_shell_err(error: io::Error, name: String) -> ShellErrorKind {
    match error.kind() {
        io::ErrorKind::NotFound => ShellErrorKind::FileNotFound(name),
        io::ErrorKind::PermissionDenied => ShellErrorKind::FilePermissionDenied(name),
        _ => ShellErrorKind::Io(None, error),
    }
}
