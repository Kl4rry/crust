use std::io::Write;

use crate::{parser::runtime_error::RunTimeError, shell::Shell};

pub fn _unalias(
    shell: &mut Shell,
    args: &[String],
    _: &mut dyn Write,
) -> Result<i64, RunTimeError> {
    let matches = clap::App::new("unalias")
        .about("set alias")
        .arg(
            clap::Arg::with_name("all")
                .short("a")
                .help("Clear all alias"),
        )
        .arg(
            clap::Arg::with_name("NAME")
                .help("Name of the alias")
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
    shell.aliases.remove(name);

    Ok(0)
}
