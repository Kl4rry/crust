use std::io::Write;

use crate::{parser::runtime_error::RunTimeError, shell::Shell};

pub fn drop(shell: &mut Shell, args: &[String], _: &mut dyn Write) -> Result<i64, RunTimeError> {
    let matches = clap::App::new("drop")
        .about("drop variable out of scope")
        .arg(
            clap::Arg::with_name("NAME")
                .help("The name of the variable to be dropped")
                .required(true),
        )
        .settings(&[clap::AppSettings::NoBinaryName])
        .get_matches_from_safe(args.iter())?;

    let name = matches.value_of("NAME").unwrap();

    let value = shell.variables.remove(name);

    if value.is_some() {
        Ok(0)
    } else {
        eprintln!("drop: variable: {} not found", name);
        Ok(-1)
    }
}
