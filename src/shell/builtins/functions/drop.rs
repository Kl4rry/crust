use std::io::Write;

use crate::{parser::shell_error::ShellErrorKind, shell::Shell};

pub fn _drop(
    shell: &mut Shell,
    args: &[String],
    _: &mut dyn Write,
) -> Result<i128, ShellErrorKind> {
    let matches = clap::App::new("drop")
        .about("drop variable out of scope")
        .arg(
            clap::Arg::new("NAME")
                .help("The name of the variable to be dropped")
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

    for frame in shell.stack.iter_mut() {
        let value = frame.variables.remove(name);
        if value.is_some() {
            return Ok(0);
        }
    }
    Ok(-1)
}
