use indexmap::IndexMap;
use once_cell::sync::Lazy;

use crate::{
    argparse::{App, ParseResult},
    parser::shell_error::ShellErrorKind,
    shell::{
        frame::Frame,
        stream::{OutputStream, ValueStream},
        value::{table::Table, SpannedValue, Value},
        Shell,
    },
};

static APP: Lazy<App> = Lazy::new(|| App::new("env").about("List all environment variables"));

pub fn env(
    _: &mut Shell,
    frame: &mut Frame,
    args: Vec<SpannedValue>,
    _: ValueStream,
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

    let mut table = Table::new();

    for (name, value) in frame.env() {
        let map = IndexMap::from([
            (String::from("Name"), Value::from(name)),
            (String::from("Value"), Value::from(value)),
        ]);
        table.insert_map(map);
    }
    output.push(Value::from(table));

    Ok(())
}
