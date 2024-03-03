use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Arg, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::value::{SpannedValue, Type},
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("assert")
        .about("Assert that something is true")
        .arg(
            Arg::new("EXPR", Type::BOOL)
                .help("Value that will be asserted")
                .required(true),
        )
});

pub fn assert(ctx: &mut Context, args: Vec<SpannedValue>) -> Result<(), ShellErrorKind> {
    let matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info)?;
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let expr = matches.value("EXPR").unwrap();
    if !expr.value.truthy() {
        return Err(ShellErrorKind::AssertionFailed(expr.span));
    }

    Ok(())
}
