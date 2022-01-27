use std::io::Write;

use crate::{parser::runtime_error::RunTimeError, shell::Shell};

pub fn _size(_: &mut Shell, args: &[String], out: &mut dyn Write) -> Result<i64, RunTimeError> {
    let matches = clap::App::new("size")
        .about("print size of terminal window")
        .settings(&[clap::AppSettings::NoBinaryName])
        .get_matches_from_safe(args.iter());

    let _ = match matches {
        Ok(matches) => matches,
        Err(clap::Error { message, .. }) => {
            eprintln!("{}", message);
            return Ok(-1);
        }
    };

    let (w, h) = crossterm::terminal::size().unwrap();
    writeln!(out, "{} {}", w, h)?;
    Ok(0)
}
