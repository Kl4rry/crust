use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Arg, ParseResult},
    parser::shell_error::ShellErrorKind,
    shell::{
        frame::Frame,
        stream::{OutputStream, ValueStream},
        value::{SpannedValue, Type},
        Shell,
    },
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

pub fn assert(
    _: &mut Shell,
    _: &mut Frame,
    args: Vec<SpannedValue>,
    _: ValueStream,
    output: &mut OutputStream,
) -> Result<(), ShellErrorKind> {
    let matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            output.push(info);
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
