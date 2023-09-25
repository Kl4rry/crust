use indexmap::IndexMap;
use once_cell::sync::Lazy;

use crate::{
    argparse::{App, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::{
        stream::ValueStream,
        value::{table::Table, SpannedValue, Value},
    },
};

static APP: Lazy<App> = Lazy::new(|| App::new("env").about("List all environment variables"));

pub fn env(
    ctx: &mut Context,
    args: Vec<SpannedValue>,
    _: ValueStream,
) -> Result<(), ShellErrorKind> {
    let _ = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let mut table = Table::new();

    for (name, value) in ctx.frame.env() {
        let map = IndexMap::from([
            ("Name".into(), Value::from(name)),
            ("Value".into(), Value::from(value)),
        ]);
        table.insert_map(map);
    }
    ctx.output.push(Value::from(table));

    Ok(())
}
