#![feature(type_alias_impl_trait)]
#![feature(once_cell)]
use std::{fs, path::PathBuf};

use clap::{Arg, Command};
mod shell;
pub use shell::parser;
use shell::{parser::shell_error::ShellErrorKind, stream::OutputStream, Shell};
mod argparse;

fn main() {
    let status = match start() {
        Ok(status) => status,
        Err(err) => {
            eprintln!("{}", err);
            match err {
                ShellErrorKind::Io(_, err) => err.raw_os_error().unwrap_or(-1),
                _ => -1,
            }
        }
    };
    std::process::exit(status);
}

fn start() -> Result<i32, ShellErrorKind> {
    let matches = Command::new(env!("CARGO_BIN_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("A exotic shell")
        .arg(Arg::new("FILE").help("Script file path"))
        .arg(
            Arg::new("ARGS")
                .help("Args to be passed to the script")
                .takes_value(true)
                .multiple_values(true),
        )
        .arg(
            Arg::new("COMMAND")
                .short('c')
                .long("command")
                .help("Command to be ran")
                .conflicts_with("FILE")
                .conflicts_with("ARGS")
                .takes_value(true)
                .required(false),
        )
        .get_matches();

    let args: Vec<_> = match matches.values_of("ARGS") {
        Some(args) => args.into_iter().map(|s| s.to_string()).collect(),
        None => Vec::new(),
    };

    let mut shell = Shell::new(args);
    if !matches.value_of("FILE").is_some() && !matches.value_of("COMMAND").is_some() {
        print!("{}", ansi_escapes::ClearScreen);
    }
    shell.init()?;

    let status = match matches.value_of("FILE") {
        Some(input) => shell.run_src(
            fs::read_to_string(input)
                .map_err(|e| ShellErrorKind::Io(Some(PathBuf::from(input)), e))?,
            String::from(input),
            &mut OutputStream::new_output(),
        ),
        None => match matches.value_of("COMMAND") {
            Some(command) => shell.run_src(
                command.to_string(),
                String::from("shell"),
                &mut OutputStream::new_output(),
            ),
            None => shell.run()?,
        },
    };

    Ok(num_traits::clamp(status, i32::MIN as i128, i32::MAX as i128) as i32)
}
