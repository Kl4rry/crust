use crate::{
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::{
        stream::ValueStream,
        value::{SpannedValue, Value},
    },
};

pub fn echo(
    ctx: &mut Context,
    args: Vec<SpannedValue>,
    _: ValueStream,
) -> Result<(), ShellErrorKind> {
    if args.len() == 1 {
        for arg in args {
            ctx.output.push(arg.into());
        }
    } else {
        ctx.output.push(Value::from(
            args.into_iter().map(|v| v.value).collect::<Vec<_>>(),
        ))
    }
    Ok(())
}
