use crate::{
    parser::shell_error::ShellErrorKind,
    shell::{
        frame::Frame,
        stream::{OutputStream, ValueStream},
        value::Value,
        Shell,
    },
};

pub fn help(
    _: &mut Shell,
    _: &mut Frame,
    _: Vec<Value>,
    _: ValueStream,
    output: &mut OutputStream,
) -> Result<(), ShellErrorKind> {
    output.push(Value::from(
        "For now you're just gonna have to figure it out :/",
    ));
    Ok(())
}
