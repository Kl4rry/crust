use crate::{
    parser::shell_error::ShellErrorKind,
    shell::{
        frame::Frame,
        stream::{OutputStream, ValueStream},
        value::{SpannedValue, Value},
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
    if args.len() == 1 {
        for arg in args {
            output.push(arg.into());
        }
    } else {
        output.push(Value::from(
            args.into_iter().map(|v| v.value).collect::<Vec<_>>(),
        ))
    }
    Ok(())
}
