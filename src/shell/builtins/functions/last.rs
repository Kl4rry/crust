use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Arg, ParseResult},
    parser::shell_error::ShellErrorKind,
    shell::{
        frame::Frame,
        stream::{OutputStream, ValueStream},
        value::{SpannedValue, Type, Value},
        Shell,
    },
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("last")
        .arg(Arg::new("COUNT", Type::INT).help("Number of items to get"))
        .about("Get last n item of sequence")
});

pub fn last(
    _: &mut Shell,
    _: &mut Frame,
    args: Vec<SpannedValue>,
    input: ValueStream,
    output: &mut OutputStream,
) -> Result<(), ShellErrorKind> {
    let matches = match APP.parse(args.into_iter().map(|v| v.into())) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let count = matches.value("COUNT").map(|v| v.unwrap_int()).unwrap_or(1);
    if count < 0 {
        return Err(ShellErrorKind::NegativeIndex { index: count });
    }
    let count = count as usize;

    let input = input.unpack();
    let last: Vec<_> = match input {
        Value::List(ref list) => list.iter().rev().take(count).cloned().collect(),
        Value::String(ref string) => string.chars().rev().take(count).map(Value::from).collect(),
        Value::Table(ref table) => table
            .rows()
            .iter()
            .rev()
            .take(count)
            .map(|v| Value::from(v.to_vec()))
            .collect(),
        Value::Range(ref range) => (**range)
            .clone()
            .rev()
            .take(count)
            .map(Value::from)
            .collect(),
        _ => {
            return Err(ShellErrorKind::Basic(
                "TypeError",
                format!("Cannot get last of {}", input.to_type()),
            ))
        }
    };

    output.extend(last.into_iter().rev());
    Ok(())
}
