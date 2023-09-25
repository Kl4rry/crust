use crate::{
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::{
        stream::ValueStream,
        value::{SpannedValue, Value},
    },
};

pub fn help(ctx: &mut Context, _: Vec<SpannedValue>, _: ValueStream) -> Result<(), ShellErrorKind> {
    ctx.output.push(Value::from(
        "For now you're just gonna have to figure it out :/",
    ));
    Ok(())
}
