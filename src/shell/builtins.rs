use std::io::{stdout, Write};

use phf::*;

use crate::{
    parser::runtime_error::RunTimeError,
    shell::{values::Value, Shell},
};

mod alias;
mod cd;
mod clear;
mod drop;
mod echo;
mod env;
mod exit;
#[cfg(target_family = "windows")]
mod ls;
mod pwd;
mod size;
mod unalias;

type Bulitin = fn(&mut Shell, &[String], &mut dyn Write) -> Result<i64, RunTimeError>;

static BUILTINS: phf::Map<&'static str, Bulitin> = phf_map! {
    "clear" => clear::clear,
    "pwd" => pwd::pwd,
    "size" => size::size,
    "exit" => exit::exit,
    "echo" => echo::echo,
    "cd" => cd::cd,
    "alias" => alias::alias,
    "unalias" => unalias::unalias,
    #[cfg(target_family = "windows")]
    "ls" => ls::ls,
    "drop" => drop::drop,
    "env" => env::env,
};

pub fn run_builtin(
    shell: &mut Shell,
    command: &str,
    args: &[String],
) -> Option<Result<Value, RunTimeError>> {
    let mut out = stdout();
    let status = match BUILTINS.get(command) {
        Some(cmd) => match cmd(shell, args, &mut out) {
            Ok(status) => status,
            Err(error) => return Some(Err(error)),
        },
        None => return None,
    };
    Some(Ok(Value::Int(status)))
}
