use std::io::Write;

use thin_string::ToThinString;

use crate::{
    parser::runtime_error::RunTimeError,
    shell::{gc::Value, Shell},
};

pub fn exit(shell: &mut Shell, args: &[String], _: &mut dyn Write) -> Result<i64, RunTimeError> {
    let matches = clap::App::new("exit")
        .about("exit the shell")
        .arg(clap::Arg::with_name("STATUS").help("The exit status of the shell"))
        .settings(&[clap::AppSettings::NoBinaryName])
        .get_matches_from_safe(args.iter())?;

    if let Some(status) = matches.value_of("STATUS") {
        shell.exit_status = match Value::String(status.to_thin_string()).try_to_int() {
            Ok(number) => number,
            Err(_) => {
                eprintln!("exit: STATUS must be integer");
                return Ok(-1);
            }
        };
    }

    shell.running = false;
    Err(RunTimeError::Exit)
}
