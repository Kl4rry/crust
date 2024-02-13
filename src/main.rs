#![feature(impl_trait_in_assoc_type)]
#![feature(get_mut_unchecked)]
use std::{
    env, fs, io,
    io::{IsTerminal, Read},
    path::{Path, PathBuf},
    process::ExitCode,
    rc::Rc,
};

use argparse::{App, Arg, Flag, Opt, ParseResult};

use crate::shell::value::Value;
mod shell;
pub use shell::parser;
use shell::{
    parser::{lexer::token::span::Span, shell_error::ShellErrorKind},
    stream::{OutputStream, ValueStream},
    value::Type,
    Shell,
};
mod argparse;
mod str_ext;
mod test;

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
    let mut input_value = Value::Null;
    if !io::stdin().is_terminal() {
        let mut buf = Vec::new();
        io::stdin()
            .read_to_end(&mut buf)
            .map_err(|e| ShellErrorKind::Io(None, e))?;
        match String::from_utf8(buf) {
            Ok(string) => input_value = Value::from(string),
            Err(e) => input_value = Value::from(e.into_bytes()),
        }
    }

    let mut args_iter = env::args();
    args_iter.next();

    let app = App::new(env!("CARGO_BIN_NAME"))
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
        .opt(
            Opt::new("PROFILE", Type::STRING)
                .long("profile")
                .help("Collect profiling data"),
        )
        .flag(
            Flag::new("CHECK")
                .long("check")
                .help("Check for syntax errors"),
        )
        .sub_cmd(App::new("license").about("View third party licenses"));

    let matches = app.parse(args_iter.map(|s| Value::String(Rc::new(s)).spanned(Span::new(0, 0))));

    let matches = match matches {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            println!("{}", info.unwrap_string());
            return Ok(ExitCode::SUCCESS);
        }
        Err(e) => return Err(e.into()),
    };

    if let Some(path) = matches.get_str("PROFILE") {
        setup_global_subscriber(path);
    }

    if matches.sub_cmd() == Some("license") {
        let licenses: &str = include_str!(concat!(env!("OUT_DIR"), "/license.html"));
        let temp = env::temp_dir();
        let name = env!("CARGO_BIN_NAME");
        fs::create_dir_all(temp.join(name))
            .map_err(|e| ShellErrorKind::Io(Some(temp.join(name)), e))?;
        let license_file = temp.join(name).join("license.html");
        fs::write(&license_file, licenses.as_bytes())
            .map_err(|e| ShellErrorKind::Io(Some(license_file.clone()), e))?;
        opener::open_browser(&license_file)?;
        return Ok(ExitCode::SUCCESS);
    }

    if let Some(path) = matches.get_str("PATH") {
        std::env::set_current_dir(path)
            .map_err(|e| ShellErrorKind::Io(Some(Path::new(path).to_path_buf()), e))?;
    }

    let mut args = Vec::new();
    args.push(env::args().next().unwrap());
    if let Some(a) = matches.get("ARGS") {
        args.extend(a.iter().map(|s| s.value.unwrap_as_str().to_string()));
    }

    let mut shell = Shell::new(args);
    let interactive = !(matches.get_str("FILE").is_some() || matches.get_str("COMMAND").is_some());
    shell.set_interactive(interactive);
    shell.init()?;

    let check = matches.conatins("CHECK");

    let (src_name, src) = if let Some(command) = matches.get_str("COMMAND") {
        (String::from("shell"), command.to_string())
    } else if let Some(file) = matches.get_str("FILE") {
        (
            String::from(file),
            fs::read_to_string(file)
                .map_err(|e| ShellErrorKind::Io(Some(PathBuf::from(file)), e))?,
        )
    } else if check {
        eprintln!("Check must be used with a file or a command");
        return Ok(ExitCode::FAILURE);
    } else {
        return Ok(ExitCode::from(
            shell.run()?.clamp(u8::MIN as i64, u8::MAX as i64) as u8,
        ));
    };

    let status = if check {
        match shell.validate_syntax(src_name, src) {
            true => ExitCode::SUCCESS,
            false => ExitCode::FAILURE,
        }
    } else {
        shell.run_src(
            src_name,
            src,
            &mut OutputStream::new_output(),
            ValueStream::from_value(input_value),
        );
        ExitCode::from(shell.status().clamp(u8::MIN as i64, u8::MAX as i64) as u8)
    };

    Ok(status)
}

fn setup_global_subscriber(path: impl AsRef<Path>) -> impl Drop {
    use tracing_flame::FlameLayer;
    use tracing_subscriber::{fmt, prelude::*, registry::Registry};
    let fmt_layer = fmt::Layer::default();

    let (flame_layer, _guard) = FlameLayer::with_file(path).unwrap();

    let subscriber = Registry::default().with(fmt_layer).with(flame_layer);

    tracing::subscriber::set_global_default(subscriber).expect("Could not set global default");
    _guard
}
