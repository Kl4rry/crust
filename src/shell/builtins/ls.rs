use std::{
    io::Write,
    process::{Command, Stdio},
};

use crate::{parser::runtime_error::RunTimeError, shell::Shell};

pub fn ls(_: &mut Shell, args: &[String], out: &mut dyn Write) -> Result<i64, RunTimeError> {
    //pwsh.exe -c "ls"
    let mut cmd_args = vec![String::from("-c"), String::from("ls")];
    cmd_args.extend_from_slice(args);
    let output = Command::new("powershell.exe")
        .args(cmd_args)
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    let string = String::from_utf8_lossy(&output.stdout);
    write!(out, "{}", string)?;
    out.flush()?;

    Ok(0)
}
