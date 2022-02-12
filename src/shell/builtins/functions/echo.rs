use crate::{
    parser::shell_error::ShellError,
    shell::{
        stream::{OutputStream, ValueStream},
        value::Value,
        Shell,
    },
};

pub fn echo(_: &mut Shell, args: &[String], _: ValueStream) -> Result<OutputStream, ShellError> {
    let mut output = OutputStream::default();
    for arg in args {
        output.stream.push(Value::String(arg.to_string()));
    }
    Ok(output)
}
