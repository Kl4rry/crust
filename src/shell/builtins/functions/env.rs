use std::io::Write;

use crate::{parser::shell_error::ShellErrorKind, shell::Shell};

pub fn _env(_: &mut Shell, args: &[String], out: &mut dyn Write) -> Result<i128, ShellErrorKind> {
    let matches = clap::Command::new("env")
        .about("List all environment variable")
        .no_binary_name(true)
        .try_get_matches_from(args.iter());

    let _ = match matches {
        Ok(matches) => matches,
        Err(err) => {
            eprintln!("{}", err);
            return Ok(-1);
        }
    };

    for (key, value) in std::env::vars() {
        writeln!(out, "{}={}", key, value).map_err(|err| ShellErrorKind::Io(None, err))?;
    }
    out.flush().map_err(|err| ShellErrorKind::Io(None, err))?;

    Ok(0)
}
