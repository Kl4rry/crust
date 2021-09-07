use std::io::Write;

use crate::{parser::runtime_error::RunTimeError, shell::Shell};

pub fn env(_: &mut Shell, args: &[String], out: &mut dyn Write) -> Result<i64, RunTimeError> {
    let _ = clap::App::new("env")
        .about("List all environment variable")
        .settings(&[clap::AppSettings::NoBinaryName])
        .get_matches_from_safe(args.iter())?;

    for (key, value) in std::env::vars() {
        writeln!(out, "{}={}", key, value)?;
    }
    out.flush()?;

    Ok(0)
}
