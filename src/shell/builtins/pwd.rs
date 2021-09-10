use std::io::Write;

use crate::{
    parser::runtime_error::RunTimeError,
    shell::{dir, Shell},
};

pub fn pwd(_: &mut Shell, args: &[String], out: &mut dyn Write) -> Result<i64, RunTimeError> {
    let matches = clap::App::new("pwd")
        .about("print working directory")
        .settings(&[clap::AppSettings::NoBinaryName])
        .get_matches_from_safe(args.iter());

    let _ = match matches {
        Ok(matches) => matches,
        Err(clap::Error { message, .. }) => {
            eprintln!("{}", message);
            return Ok(-1);
        }
    };

    writeln!(out, "{}", dir().to_string_lossy())?;
    Ok(0)
}
