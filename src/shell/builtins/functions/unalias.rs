use std::io::Write;

use crate::{parser::runtime_error::RunTimeError, shell::Shell};

pub fn _unalias(
    shell: &mut Shell,
    args: &[String],
    _: &mut dyn Write,
) -> Result<i128, RunTimeError> {
    let matches = clap::App::new("unalias")
        .about("set alias")
        .arg(clap::Arg::new("all").short('a').help("Clear all alias"))
        .arg(
            clap::Arg::new("NAME")
                .help("Name of the alias")
                .required(true),
        )
        .setting(clap::AppSettings::NoBinaryName)
        .try_get_matches_from(args.iter());

    let matches = match matches {
        Ok(matches) => matches,
        Err(err) => {
            eprintln!("{}", err);
            return Ok(-1);
        }
    };

    let name = matches.value_of("NAME").unwrap();
    shell.aliases.remove(name);

    Ok(0)
}
