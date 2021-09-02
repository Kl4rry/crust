use std::{io::Write, path::Path};

use crossterm::{execute, style::Print};
use thin_string::ToThinString;

use super::{clear_str, dir, Shell};
use crate::{parser::runtime_error::RunTimeError, shell::gc::Value};

pub fn run_builtin(
    shell: &mut Shell,
    command: &str,
    args: &[String],
) -> Option<Result<Value, RunTimeError>> {
    match command {
        "clear" => Some(clear(shell)),
        "pwd" => Some(pwd()),
        "size" => Some(size()),
        "exit" => Some(exit(shell, args)),
        "echo" => Some(echo(shell, args)),
        "cd" => Some(cd(shell, args)),
        "alias" => Some(alias(shell, args)),
        "unalias" => Some(unalias(shell, args)),
        _ => None,
    }
}

pub fn clear(shell: &mut Shell) -> Result<Value, RunTimeError> {
    //https://superuser.com/questions/1628694/how-do-i-add-a-keyboard-shortcut-to-clear-scrollback-buffer-in-windows-terminal
    (execute! {
        shell.stdout,
        Print(clear_str()),
    })?;
    Ok(Value::ExitStatus(0))
}

pub fn pwd() -> Result<Value, RunTimeError> {
    println!("{}", dir().to_string_lossy());
    Ok(Value::ExitStatus(0))
}

pub fn size() -> Result<Value, RunTimeError> {
    let (w, h) = crossterm::terminal::size().unwrap();
    println!("{} {}", w, h);
    Ok(Value::ExitStatus(0))
}

pub fn exit(shell: &mut Shell, args: &[String]) -> Result<Value, RunTimeError> {
    let matches = clap::App::new("exit")
        .about("exit the shell")
        .arg(clap::Arg::with_name("STATUS").help("The exit status of the shell"))
        .settings(&[clap::AppSettings::NoBinaryName])
        .get_matches_from_safe(args.iter())?;

    if let Some(status) = matches.value_of("STATUS") {
        shell.exit_status = match Value::String(status.to_thin_string()).try_to_int() {
            Ok(number) => number,
            Err(_) => {
                eprintln!("exit: STATUS must be integer");
                return Ok(Value::ExitStatus(-1));
            }
        };
    }

    shell.running = false;
    Err(RunTimeError::Exit)
}

pub fn echo(shell: &mut Shell, args: &[String]) -> Result<Value, RunTimeError> {
    for arg in args {
        write!(shell.stdout, "{} ", arg)?;
    }
    write!(shell.stdout, "\n")?;
    shell.stdout.flush()?;
    Ok(Value::ExitStatus(0))
}

pub fn cd(shell: &mut Shell, args: &[String]) -> Result<Value, RunTimeError> {
    let matches = clap::App::new("cd")
        .about("change directory")
        .arg(clap::Arg::with_name("DIR").help("The new directory"))
        .settings(&[clap::AppSettings::NoBinaryName])
        .get_matches_from_safe(args.iter())?;

    let dir = match matches.value_of("DIR") {
        Some(value) => value,
        None => shell.home_dir.to_str().unwrap(),
    };

    let new_dir = Path::new(dir);
    if let Err(e) = std::env::set_current_dir(&new_dir) {
        eprintln!("{}", e);
    }
    Ok(Value::ExitStatus(0))
}

pub fn alias(shell: &mut Shell, args: &[String]) -> Result<Value, RunTimeError> {
    let matches = clap::App::new("alias")
        .about("set alias")
        .arg(
            clap::Arg::with_name("NAME")
                .help("The new directory")
                .required(true),
        )
        .arg(
            clap::Arg::with_name("COMMAND")
                .help("The new directory")
                .required(true),
        )
        .settings(&[clap::AppSettings::NoBinaryName])
        .get_matches_from_safe(args.iter())?;

    let name = matches.value_of("NAME").unwrap();
    let command = matches.value_of("COMMAND").unwrap();

    if name.len() < 1 {
        eprintln!("alias: NAME must be atleast on character long");
        return Ok(Value::ExitStatus(-1));
    }

    if command.len() < 1 {
        eprintln!("alias: COMMAND must be atleast on character long");
        return Ok(Value::ExitStatus(-1));
    }

    shell.aliases.insert(name.to_string(), command.to_string());
    Ok(Value::ExitStatus(0))
}

pub fn unalias(shell: &mut Shell, args: &[String]) -> Result<Value, RunTimeError> {
    let matches = clap::App::new("unalias")
        .about("set alias")
        .arg(
            clap::Arg::with_name("NAME")
                .help("The new directory")
                .required(true),
        )
        .settings(&[clap::AppSettings::NoBinaryName])
        .get_matches_from_safe(args.iter())?;

    let name = matches.value_of("NAME").unwrap();
    shell.aliases.remove(name);

    Ok(Value::ExitStatus(0))
}
