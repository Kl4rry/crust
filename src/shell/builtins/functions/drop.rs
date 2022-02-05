use std::io::Write;

use crate::{parser::runtime_error::RunTimeError, shell::Shell};

pub fn _drop(shell: &mut Shell, args: &[String], _: &mut dyn Write) -> Result<i128, RunTimeError> {
    let matches = clap::App::new("drop")
        .about("drop variable out of scope")
        .arg(
            clap::Arg::with_name("NAME")
                .help("The name of the variable to be dropped")
                .required(true),
        )
        .settings(&[clap::AppSettings::NoBinaryName])
        .get_matches_from_safe(args.iter());

    let matches = match matches {
        Ok(matches) => matches,
        Err(clap::Error { message, .. }) => {
            eprintln!("{}", message);
            return Ok(-1);
        }
    };

    let name = matches.value_of("NAME").unwrap();

    for frame in shell.stack.iter_mut() {
        let value = frame.variables.remove(name);
        if value.is_some() {
            return Ok(0);
        }
    }
    Ok(-1)
}
