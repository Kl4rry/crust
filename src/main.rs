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
        .get_matches();

    let shell = Shell::new();
    let status = match matches.value_of("INPUT") {
        Some(input) => shell.run_src(fs::read_to_string(input)?),
        None => shell.run(),
    };
    std::process::exit(status as i32);
}
