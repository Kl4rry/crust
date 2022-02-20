use crate::{
    parser::shell_error::ShellErrorKind,
    shell::{
        stream::{OutputStream, ValueStream},
        value::Value,
        Shell,
    },
};

pub fn echo(
    _: &mut Shell,
    args: Vec<Value>,
    _: ValueStream,
) -> Result<OutputStream, ShellErrorKind> {
    let mut output = OutputStream::default();
    for arg in args {
        output.push(arg);
    }
    Ok(output)
}
