use thin_string::ThinString;

use crate::{
    parser::runtime_error::RunTimeError,
    shell::{
        clear_str,
        stream::{OutputStream, ValueStream},
        value::Value,
        Shell,
    },
};

pub fn clear(_: &mut Shell, _: &[String], _: ValueStream) -> Result<OutputStream, RunTimeError> {
    //https://superuser.com/questions/1628694/how-do-i-add-a-keyboard-shortcut-to-clear-scrollback-buffer-in-windows-terminal
    let stream = ValueStream::from_value(Value::String(ThinString::from(clear_str())));
    Ok(OutputStream { stream, status: 0 })
}
