use phf::*;

use crate::{
    parser::runtime_error::RunTimeError,
    shell::{
        stream::{OutputStream, ValueStream},
        Shell,
    },
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

pub type BulitinFn = fn(&mut Shell, &[String], ValueStream) -> Result<OutputStream, RunTimeError>;

static BUILTIN_FUNCTIONS: phf::Map<&'static str, BulitinFn> = phf_map! {
    "clear" => clear::clear,
    "pwd" => pwd::pwd,
    "exit" => exit::exit,
    /*"size" => size::size,
    "echo" => echo::echo,
    "cd" => cd::cd,
    "alias" => alias::alias,
    "unalias" => unalias::unalias,
    "drop" => drop::drop,
    "env" => env::env,*/
};

pub fn get_builtin(command: &str) -> Option<BulitinFn> {
    BUILTIN_FUNCTIONS.get(command).map(|f| f.clone())
}
