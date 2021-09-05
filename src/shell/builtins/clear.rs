use std::io::Write;

use crate::{
    parser::runtime_error::RunTimeError,
    shell::{clear_str, Shell},
};

pub fn clear(_: &mut Shell, _: &[String], out: &mut dyn Write) -> Result<i64, RunTimeError> {
    //https://superuser.com/questions/1628694/how-do-i-add-a-keyboard-shortcut-to-clear-scrollback-buffer-in-windows-terminal
    write!(out, "{}", clear_str())?;
    out.flush()?;
    Ok(0)
}
