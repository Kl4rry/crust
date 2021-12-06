use std::{io::Write, path::Path};

use crate::{parser::runtime_error::RunTimeError, shell::Shell};

pub fn cd(shell: &mut Shell, args: &[String], _: &mut dyn Write) -> Result<i64, RunTimeError> {
    let matches = clap::App::new("cd")
        .about("change directory")
        .arg(clap::Arg::with_name("DIR").help("The new directory"))
        .settings(&[clap::AppSettings::NoBinaryName])
        .get_matches_from_safe(args.iter());

    let matches = match matches {
        Ok(matches) => matches,
        Err(clap::Error { message, .. }) => {
            eprintln!("{}", message);
            return Ok(-1);
        }
    };

    let dir = match matches.value_of("DIR") {
        Some(value) => value,
        None => shell.home_dir.to_str().unwrap(),
    };

    let new_dir = Path::new(dir);
    if let Err(e) = std::env::set_current_dir(&new_dir) {
        eprintln!("{}", e);
    }
    Ok(0)
}
