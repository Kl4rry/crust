use std::io::{stdout, Write};

use phf::*;

use crate::{
    parser::runtime_error::RunTimeError,
    shell::{value::Value, Shell},
};

mod alias;
mod cd;
mod clear;
mod drop;
mod echo;
mod env;
mod exit;
mod pwd;
mod size;
mod unalias;

type BulitinFn = fn(&mut Shell, &[String], &mut dyn Write) -> Result<i64, RunTimeError>;

static BUILTIN_FUNCTIONS: phf::Map<&'static str, BulitinFn> = phf_map! {
    "clear" => clear::clear,
    "pwd" => pwd::pwd,
    "size" => size::size,
    "exit" => exit::exit,
    "echo" => echo::echo,
    "cd" => cd::cd,
    "alias" => alias::alias,
    "unalias" => unalias::unalias,
    "drop" => drop::drop,
    "env" => env::env,
};

pub fn run_builtin(
    shell: &mut Shell,
    command: &str,
    args: &[String],
) -> Option<Result<Value, RunTimeError>> {
    let status = match BUILTIN_FUNCTIONS.get(command) {
        Some(cmd) => match cmd(shell, args, &mut stdout()) {
            Ok(status) => status,
            Err(error) => return Some(Err(error)),
        },
        None => return None,
    };
    Some(Ok(Value::Int(status)))
}
