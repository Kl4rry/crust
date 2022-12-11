use crate::{
    parser::shell_error::ShellErrorKind,
    shell::{
        frame::Frame,
        stream::{OutputStream, ValueStream},
        value::SpannedValue,
        Shell,
    },
};

pub fn echo(
    _: &mut Shell,
    _: &mut Frame,
    args: Vec<SpannedValue>,
    _: ValueStream,
    output: &mut OutputStream,
) -> Result<(), ShellErrorKind> {
    for arg in args {
        output.push(arg.into());
    }
    Ok(())
}
