#![feature(drain_filter)]
#![feature(type_alias_impl_trait)]
use std::fs;

use clap::{App, Arg};
mod shell;
pub use shell::parser;
use shell::Shell;

fn main() -> Result<(), std::io::Error> {
    let matches = App::new(env!("CARGO_BIN_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("A exotic shell")
        .arg(
            Arg::with_name("INPUT")
                .help("Input script file")
                .required(false),
        )
        .arg(
            Arg::with_name("COMMAND")
                .short("c")
                .long("command")
                .help("Command")
                .conflicts_with("INPUT")
                .takes_value(true)
                .required(false),
        )
        .get_matches();

    let shell = Shell::new();

    let status = match matches.value_of("INPUT") {
        Some(input) => shell.run_src(fs::read_to_string(input)?, String::from(input)),
        None => match matches.value_of("COMMAND") {
            Some(command) => shell.run_src(command.to_string(), String::from("shell")),
            None => shell.run(),
        },
    };

    std::process::exit(status as i32);
}
