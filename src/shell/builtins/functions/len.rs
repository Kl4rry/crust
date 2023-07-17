use once_cell::sync::Lazy;

use crate::{
    argparse::{App, ParseResult},
    parser::shell_error::ShellErrorKind,
    shell::{
        frame::Frame,
        stream::{OutputStream, ValueStream},
        value::{SpannedValue, Value},
        Shell,
    },
};

static APP: Lazy<App> = Lazy::new(|| App::new("len").about("Get number of items in container"));

pub fn len(
    _: &mut Shell,
    _: &mut Frame,
    args: Vec<SpannedValue>,
    input: ValueStream,
    output: &mut OutputStream,
) -> Result<(), ShellErrorKind> {
    let _ = match APP.parse(args.into_iter().map(|v| v.into())) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let input = input.unpack();
    let len = match input {
        Value::String(string) => string.chars().count() as i64,
        Value::List(list) => list.len() as i64,
        Value::Map(map) => map.len() as i64,
        Value::Table(table) => table.len() as i64,
        Value::Range(range) => (range.end - range.start).max(0),
        _ => {
            return Err(ShellErrorKind::Basic(
                "TypeError",
                format!("Cannot get length of `{}`", input.to_type()),
            ))
        }
    };

    output.push(len.into());

    Ok(())
}
