use std::io::Write;

use crate::{parser::runtime_error::RunTimeError, shell::Shell};

pub fn _env(_: &mut Shell, args: &[String], out: &mut dyn Write) -> Result<i64, RunTimeError> {
    let matches = clap::App::new("env")
        .about("List all environment variable")
        .settings(&[clap::AppSettings::NoBinaryName])
        .get_matches_from_safe(args.iter());

    let _ = match matches {
        Ok(matches) => matches,
        Err(clap::Error { message, .. }) => {
            eprintln!("{}", message);
            return Ok(-1);
        }
    };

    for (key, value) in std::env::vars() {
        writeln!(out, "{}={}", key, value)?;
    }
    out.flush()?;

    Ok(0)
}
