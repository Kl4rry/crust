use std::io::{stdout, Write};

use crate::{
    parser::runtime_error::RunTimeError,
    shell::{
        clear_str,
        stream::{OutputStream, ValueStream},
        Shell,
    },
};

pub fn clear(_: &mut Shell, _: &[String], _: ValueStream) -> Result<OutputStream, RunTimeError> {
    //https://superuser.com/questions/1628694/how-do-i-add-a-keyboard-shortcut-to-clear-scrollback-buffer-in-windows-terminal
    stdout().write_all(clear_str().as_bytes())?;
    Ok(OutputStream::default())
}
