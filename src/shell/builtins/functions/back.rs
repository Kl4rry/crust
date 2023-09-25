use once_cell::sync::Lazy;

use crate::{
    argparse::{App, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::{stream::ValueStream, value::SpannedValue},
};

static APP: Lazy<App> =
    Lazy::new(|| App::new("back").about("Go back to the last working directory"));

pub fn back(
    ctx: &mut Context,
    args: Vec<SpannedValue>,
    _: ValueStream,
) -> Result<(), ShellErrorKind> {
    match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    ctx.shell.dir_history.back()
}
