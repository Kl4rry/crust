use std::path::Path;

use crate::{
    parser::runtime_error::RunTimeError,
    shell::{
        stream::{OutputStream, ValueStream},
        Shell,
    },
};

pub fn cd(
    shell: &mut Shell,
    args: &[String],
    _: ValueStream,
) -> Result<OutputStream, RunTimeError> {
    let matches = clap::App::new("cd")
        .about("change directory")
        .arg(clap::Arg::new("DIR").help("The new directory"))
        .setting(clap::AppSettings::NoBinaryName)
        .try_get_matches_from(args.iter());

    let mut output = OutputStream::default();

    let matches = match matches {
        Ok(matches) => matches,
        Err(err) => {
            eprintln!("{}", err);
            output.status = -1;
            return Ok(output);
        }
    };

    let dir = match matches.value_of("DIR") {
        Some(value) => value,
        None => shell.home_dir.to_str().unwrap(),
    };

    let new_dir = Path::new(dir);
    if let Err(e) = std::env::set_current_dir(&new_dir) {
        eprintln!("{}", e);
        output.status = -1;
    }
    Ok(output)
}
