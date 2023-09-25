use once_cell::sync::Lazy;

use crate::{
    argparse::{App, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::{
        stream::ValueStream,
        value::{SpannedValue, Value},
    },
};

static APP: Lazy<App> = Lazy::new(|| App::new("len").about("Get number of items in container"));

pub fn len(
    ctx: &mut Context,
    args: Vec<SpannedValue>,
    input: ValueStream,
) -> Result<(), ShellErrorKind> {
    let _ = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info);
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
        Value::Binary(data) => data.len() as i64,
        _ => {
            return Err(ShellErrorKind::Basic(
                "TypeError",
                format!("Cannot get length of {}", input.to_type()),
            ));
        }
    };

    ctx.output.push(len.into());

    Ok(())
}
