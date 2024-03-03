use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Arg, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::value::{SpannedValue, Type, Value},
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("exit")
        .about("Exit the shell")
        .arg(Arg::new("STATUS", Type::INT).help("The exit status of the shell"))
});

pub fn exit(ctx: &mut Context, args: Vec<SpannedValue>) -> Result<(), ShellErrorKind> {
    let matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info)?;
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let status = matches
        .value("STATUS")
        .map(|value| &value.value)
        .unwrap_or(&Value::Int(0));
    ctx.shell.exit_status = status.unwrap_int();

    ctx.shell.running = false;
    Err(ShellErrorKind::Exit)
}
