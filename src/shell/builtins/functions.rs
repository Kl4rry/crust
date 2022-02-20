use phf::*;

use crate::{
    parser::shell_error::ShellErrorKind,
    shell::{
        stream::{OutputStream, ValueStream},
        value::Value,
        Shell,
    },
};

mod alias;
mod cd;
mod clear;
mod echo;
mod env;
mod exit;
mod pwd;
mod unalias;

pub type BulitinFn =
    fn(&mut Shell, Vec<Value>, ValueStream) -> Result<OutputStream, ShellErrorKind>;

static BUILTIN_FUNCTIONS: phf::Map<&'static str, BulitinFn> = phf_map! {
    "clear" => clear::clear,
    "pwd" => pwd::pwd,
    "exit" => exit::exit,
    "cd" => cd::cd,
    "echo" => echo::echo,
    /*"alias" => alias::alias,
    "unalias" => unalias::unalias,
    "env" => env::env,*/
};

pub fn get_builtin(command: &str) -> Option<BulitinFn> {
    BUILTIN_FUNCTIONS.get(command).copied()
}
