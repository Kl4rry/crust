use std::io::Write;

use crate::{parser::runtime_error::RunTimeError, shell::Shell};

pub fn echo(shell: &mut Shell, args: &[String], out: &mut dyn Write) -> Result<i64, RunTimeError> {
    for arg in args {
        write!(out, "{} ", arg)?;
    }
    writeln!(out)?;
    shell.stdout.flush()?;
    Ok(0)
}
