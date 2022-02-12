use std::{
    io::{stdout, Write},
    path::PathBuf,
};

use crate::{
    parser::shell_error::ShellError,
    shell::{
        clear_str,
        stream::{OutputStream, ValueStream},
        Shell,
    },
};

pub fn clear(_: &mut Shell, _: &[String], _: ValueStream) -> Result<OutputStream, ShellError> {
    //https://superuser.com/questions/1628694/how-do-i-add-a-keyboard-shortcut-to-clear-scrollback-buffer-in-windows-terminal
    stdout()
        .write_all(clear_str().as_bytes())
        .map_err(|err| ShellError::Io(PathBuf::from("stdout"), err))?;
    Ok(OutputStream::default())
}
