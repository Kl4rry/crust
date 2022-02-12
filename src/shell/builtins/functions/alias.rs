use std::io::Write;

use crate::{parser::shell_error::ShellError, shell::Shell};

pub fn _alias(shell: &mut Shell, args: &[String], _: &mut dyn Write) -> Result<i128, ShellError> {
    let matches = clap::App::new("alias")
        .about("set alias")
        .arg(
            clap::Arg::new("NAME")
                .help("Name of the alias")
                .required(true),
        )
        .arg(
            clap::Arg::new("COMMAND")
                .help("The command that will be run")
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
    let command = matches.value_of("COMMAND").unwrap();

    if name.is_empty() {
        eprintln!("alias: NAME must be atleast on character long");
        return Ok(-1);
    }

    if command.is_empty() {
        eprintln!("alias: COMMAND must be atleast on character long");
        return Ok(-1);
    }

    shell.aliases.insert(name.to_string(), command.to_string());
    Ok(0)
}
