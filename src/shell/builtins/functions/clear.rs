use std::io::{stdout, Write};

use crate::{
    parser::shell_error::ShellErrorKind,
    shell::{
        clear_str,
        stream::{OutputStream, ValueStream},
        Shell,
    },
};

pub fn clear(_: &mut Shell, _: &[String], _: ValueStream) -> Result<OutputStream, ShellErrorKind> {
    //https://superuser.com/questions/1628694/how-do-i-add-a-keyboard-shortcut-to-clear-scrollback-buffer-in-windows-terminal
    stdout()
        .write_all(clear_str().as_bytes())
        .map_err(|err| ShellErrorKind::Io(None, err))?;
    Ok(OutputStream::default())
}
