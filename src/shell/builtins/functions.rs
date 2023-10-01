use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use phf::*;

use crate::{
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::value::SpannedValue,
};

mod alias;
mod assert;
mod back;
mod cd;
mod clear;
mod do_closure;
mod echo;
mod env;
mod exit;
mod filter;
mod first;
mod help;
mod import;
mod input;
mod last;
mod len;
mod lines;
mod load;
mod map;
mod open;
mod print;
mod pwd;
mod save;
mod shuffle;
mod time;
mod unalias;
mod unique;

pub type BulitinFn = fn(&mut Context, Vec<SpannedValue>) -> Result<(), ShellErrorKind>;

static BUILTIN_FUNCTIONS: phf::Map<&'static str, BulitinFn> = phf_map! {
    "alias" => alias::alias,
    "assert" => assert::assert,
    "back" => back::back,
    "cd" => cd::cd,
    "clear" => clear::clear,
    "do" => do_closure::do_closure,
    "echo" => echo::echo,
    "env" => env::env,
    "exit" => exit::exit,
    "first" => first::first,
    "help" => help::help,
    "import" => import::import,
    "input" => input::input,
    "last" => last::last,
    "len" => len::len,
    "lines" => lines::lines,
    "load" => load::load,
    "map" => map::map,
    "open" => open::open,
    "print" => print::print,
    "pwd" => pwd::pwd,
    "save" => save::save,
    "shuffle" => shuffle::shuffle,
    "time" => time::time,
    "unalias" => unalias::unalias,
    "filter" => filter::filter,
    "unique" => unique::unique,
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

pub fn read_file_raw(path: impl AsRef<Path>) -> Result<Vec<u8>, ShellErrorKind> {
    let path = path.as_ref();
    fs::read(path).map_err(|e| file_err_to_shell_err(e, path.to_string_lossy().to_string()))
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
