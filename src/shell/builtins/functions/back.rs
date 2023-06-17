use once_cell::sync::Lazy;

use crate::{
    argparse::{App, ParseResult},
    parser::shell_error::ShellErrorKind,
    shell::{
        frame::Frame,
        stream::{OutputStream, ValueStream},
        value::SpannedValue,
        Shell,
    },
};

static APP: Lazy<App> =
    Lazy::new(|| App::new("back").about("Go back to the last working directory"));

pub fn back(
    _: &mut Shell,
    frame: &mut Frame,
    args: Vec<SpannedValue>,
    _: ValueStream,
    output: &mut OutputStream,
) -> Result<(), ShellErrorKind> {
    match APP.parse(args.into_iter().map(|v| v.into())) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    frame.back()
}
