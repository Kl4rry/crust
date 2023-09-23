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
    App::new("first")
        .arg(Arg::new("COUNT", Type::INT).help("Number of items to get"))
        .about("Get first item of sequence")
});

pub fn first(
    _: &mut Shell,
    _: &mut Frame,
    args: Vec<SpannedValue>,
    input: ValueStream,
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

    let count = matches
        .value("COUNT")
        .map(|v| v.value.unwrap_int())
        .unwrap_or(1);
    if count < 0 {
        return Err(ShellErrorKind::NegativeIndex { index: count });
    }
    let count = count as usize;

    let input = input.unpack();
    match input {
        Value::List(ref list) => output.extend(list.iter().take(count).cloned()),
        Value::String(ref string) => {
            output.push(string.chars().take(count).collect::<String>().into())
        }
        Value::Table(ref table) => output.extend(
            table
                .rows()
                .iter()
                .take(count)
                .map(|v| Value::from(v.to_vec())),
        ),
        Value::Range(ref range) => output.extend((**range).clone().take(count).map(Value::from)),
        Value::Binary(ref data) => output.push(Value::from(
            data.iter().copied().take(count).collect::<Vec<u8>>(),
        )),
        _ => {
            return Err(ShellErrorKind::Basic(
                "TypeError",
                format!("Cannot get first of {}", input.to_type()),
            ))
        }
    };

    Ok(())
}
