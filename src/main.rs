#![feature(type_alias_impl_trait)]
#![feature(arc_unwrap_or_clone)]
#![feature(get_mut_unchecked)]
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
    rc::Rc,
};

use argparse::{App, Arg, Opt, ParseResult};

use crate::shell::value::Value;
mod shell;
pub use shell::parser;
use shell::{parser::shell_error::ShellErrorKind, stream::OutputStream, value::Type, Shell};
mod argparse;

pub type P<T> = Box<T>;

fn main() -> ExitCode {
    match start() {
        Ok(status) => status,
        Err(err) => {
            eprintln!("{}", err);
            match err {
                ShellErrorKind::Io(_, err) => err
                    .raw_os_error()
                    .map(|err| ExitCode::from(err.clamp(u8::MIN as i32, u8::MAX as i32) as u8))
                    .unwrap_or(ExitCode::FAILURE),
                _ => ExitCode::FAILURE,
            }
        }
    }
}

fn start() -> Result<ExitCode, ShellErrorKind> {
    let mut args_iter = env::args();
    args_iter.next();

    let matches = App::new(env!("CARGO_BIN_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("An exotic shell")
        .arg(Arg::new("FILE", Type::STRING).help("Script file path"))
        .arg(
            Arg::new("ARGS", Type::STRING)
                .help("Args to be passed to the script")
                .multiple(true),
        )
        .opt(
            Opt::new("COMMAND", Type::STRING)
                .short('c')
                .long("command")
                .help("Command to be ran")
                .conflicts_with(String::from("FILE"))
                .conflicts_with(String::from("ARGS")),
        )
        .opt(
            Opt::new("PATH", Type::STRING)
                .short('p')
                .long("path")
                .help("The working directory the shell will run in"),
        )
        .parse(args_iter.map(|s| Value::String(Rc::new(s))));

    let matches = match matches {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            println!("{}", info.unwrap_string());
            return Ok(ExitCode::SUCCESS);
        }
        Err(e) => return Err(e.into()),
    };

    if let Some(path) = matches.get_str("PATH") {
        std::env::set_current_dir(path)
            .map_err(|e| ShellErrorKind::Io(Some(Path::new(path).to_path_buf()), e))?;
    }

    let args: Vec<_> = match matches.get("ARGS") {
        Some(args) => args.iter().map(|s| s.unwrap_as_str().to_string()).collect(),
        None => Vec::new(),
    };

    let mut shell = Shell::new(args);
    shell.init()?;

    let status = match matches.get_str("FILE") {
        Some(input) => {
            shell.run_src(
                fs::read_to_string(input)
                    .map_err(|e| ShellErrorKind::Io(Some(PathBuf::from(input)), e))?,
                String::from(input),
                &mut OutputStream::new_output(),
            );
            shell.status()
        }
        None => match matches.get_str("COMMAND") {
            Some(command) => {
                shell.run_src(
                    command.to_string(),
                    String::from("shell"),
                    &mut OutputStream::new_output(),
                );
                shell.status()
            }
            None => shell.run()?,
        },
    };

    Ok(ExitCode::from(
        num_traits::clamp(status, u8::MIN as i64, u8::MAX as i64) as u8,
    ))
}
