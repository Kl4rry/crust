use std::rc::Rc;

use indexmap::IndexSet;
use once_cell::sync::Lazy;

use crate::{
    argparse::{App, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::value::{SpannedValue, Value},
};

static APP: Lazy<App> = Lazy::new(|| App::new("unique").about("Unique values in sequence"));

pub fn unique(ctx: &mut Context, args: Vec<SpannedValue>) -> Result<(), ShellErrorKind> {
    let _ = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info)?;
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let value = ctx.input.take().unpack();
    let t = value.to_type();
    let value = match value {
        Value::String(string) => {
            let mut set = IndexSet::new();
            for ch in string.chars() {
                set.insert(Value::from(ch).into_hashable());
            }
            Value::from(set.into_iter().map(Value::from).collect::<Vec<_>>())
        }
        Value::List(list) => {
            let mut set = IndexSet::new();
            for value in list.iter() {
                set.insert(value.clone().into_hashable());
            }
            Value::from(set.into_iter().map(Value::from).collect::<Vec<_>>())
        }
        Value::Table(mut table) => {
            Rc::make_mut(&mut table).unique();
            Value::Table(table)
        }
        Value::Range(range) => {
            let mut set = IndexSet::new();
            for i in (*range).clone() {
                set.insert(Value::from(i).into_hashable());
            }
            Value::from(set.into_iter().map(Value::from).collect::<Vec<_>>())
        }
        Value::Binary(binary) => {
            let mut set = IndexSet::new();
            for i in binary.iter().copied() {
                set.insert(i);
            }
            Value::from(set.into_iter().collect::<Vec<_>>())
        }
        _ => {
            return Err(ShellErrorKind::Basic(
                "TypeError",
                format!("Unique does not support {}", t),
            ))
        }
    };

    ctx.output.push(value)?;
    Ok(())
}
