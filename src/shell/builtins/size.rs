use std::io::Write;

use crate::{parser::runtime_error::RunTimeError, shell::Shell};

pub fn size(_: &mut Shell, args: &[String], out: &mut dyn Write) -> Result<i64, RunTimeError> {
    let _ = clap::App::new("size")
        .about("print size of terminal window")
        .settings(&[clap::AppSettings::NoBinaryName])
        .get_matches_from_safe(args.iter())?;
    let (w, h) = crossterm::terminal::size().unwrap();
    writeln!(out, "{} {}", w, h)?;
    Ok(0)
}
