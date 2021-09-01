use std::{io::Write, path::Path};

use crossterm::{execute, style::Print};

use super::{clear_str, dir, Shell};
use crate::{parser::runtime_error::RunTimeError, shell::gc::Value};

pub fn run_builtin(
    shell: &mut Shell,
    command: &str,
    args: &Vec<String>,
) -> Option<Result<Value, RunTimeError>> {
    match command {
        "clear" => Some(clear(shell)),
        "pwd" => Some(pwd()),
        "size" => Some(size()),
        "exit" => Some(exit(shell, args)),
        "echo" => Some(echo(shell, args)),
        "cd" => Some(cd(shell, args)),
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

pub fn exit(shell: &mut Shell, args: &Vec<String>) -> Result<Value, RunTimeError> {
    let matches = clap::App::new("exit")
        .about("exit the shell")
        .arg(clap::Arg::with_name("STATUS").help("The exit status of the shell"))
        .settings(&[clap::AppSettings::NoBinaryName])
        .get_matches_from_safe(args.iter())?;

    if let Some(status) = matches.value_of("STATUS") {
        //shell.exit_status = Value::String(status.to_thin_string()).to_int();
        shell.exit_status = status.parse().unwrap();
    }

    shell.running = false;
    shell.exit_status;
    Ok(Value::ExitStatus(0))
}

pub fn echo(shell: &mut Shell, args: &Vec<String>) -> Result<Value, RunTimeError> {
    for arg in args {
        write!(shell.stdout, "{} ", arg)?;
    }
    write!(shell.stdout, "\n")?;
    shell.stdout.flush()?;
    Ok(Value::ExitStatus(0))
}

pub fn cd(shell: &mut Shell, args: &Vec<String>) -> Result<Value, RunTimeError> {
    let matches = clap::App::new("exit")
        .about("change directory")
        .arg(clap::Arg::with_name("DIR").help("The new directory"))
        .settings(&[clap::AppSettings::NoBinaryName])
        .get_matches_from_safe(args.iter())?;

    let dir = match matches.value_of("DIR") {
        Some(value) => value,
        None => shell.home_dir.to_str().unwrap(), // should be home dir
    };

    /*let new_dir = args.first().map_or("./", |x| *x);*/
    let root = Path::new(dir);
    if let Err(e) = std::env::set_current_dir(&root) {
        eprintln!("{}", e);
    }
    Ok(Value::ExitStatus(0))
}
