use std::{collections::VecDeque, rc::Rc};

use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Arg, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::value::{SpannedValue, Type, Value},
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("last")
        .arg(Arg::new("COUNT", Type::INT).help("Number of items to get"))
        .about("Get last n item of sequence")
});

pub fn last(ctx: &mut Context, args: Vec<SpannedValue>) -> Result<(), ShellErrorKind> {
    let matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info);
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

    let input = ctx.input.take().unpack();
    match input {
        Value::List(mut list) => {
            {
                let list = Rc::make_mut(&mut list);
                let removed = list.len().saturating_sub(count);
                list.drain(..removed);
            }
            ctx.output.push(Value::List(list));
        }
        Value::String(ref string) => {
            let mut buffer = VecDeque::new();
            for ch in string.chars() {
                buffer.push_back(ch);
                if buffer.len() > count {
                    buffer.pop_front();
                }
            }
            ctx.output
                .push(Value::from(buffer.into_iter().collect::<String>()));
        }
        Value::Table(mut table) => {
            Rc::make_mut(&mut table).last(count);
            ctx.output.push(Value::Table(table));
        }
        Value::Range(ref range) => {
            let mut buffer = VecDeque::new();
            for i in (**range).clone() {
                buffer.push_back(Value::from(i));
                if buffer.len() > count {
                    buffer.pop_front();
                }
            }
            ctx.output.push(Value::from(Vec::from(buffer)));
        }
        Value::Binary(mut data) => {
            {
                let data = Rc::make_mut(&mut data);
                let removed = data.len().saturating_sub(count);
                data.drain(..removed);
            }
            ctx.output.push(Value::Binary(data));
        }
        _ => {
            return Err(ShellErrorKind::Basic(
                "TypeError",
                format!("Cannot get last of {}", input.to_type()),
            ))
        }
    }

    Ok(())
}
