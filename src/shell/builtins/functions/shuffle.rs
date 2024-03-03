use std::rc::Rc;

use once_cell::sync::Lazy;
use rand::prelude::*;

use crate::{
    argparse::{App, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::value::{SpannedValue, Value},
};

static APP: Lazy<App> = Lazy::new(|| App::new("shuffle").about("Shuffle items in container"));

pub fn shuffle(ctx: &mut Context, args: Vec<SpannedValue>) -> Result<(), ShellErrorKind> {
    let _ = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info)?;
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let mut input = ctx.input.take().unpack();

    match input {
        Value::List(ref mut list) => {
            let mut rng = rand::thread_rng();
            let output = Rc::make_mut(list);
            output.shuffle(&mut rng);
        }
        Value::String(ref mut string) => {
            let mut rng = rand::thread_rng();
            let mut chars: Vec<char> = string.chars().collect();
            chars.shuffle(&mut rng);
            let output = Rc::make_mut(string);
            output.clear();
            output.extend(chars);
        }
        Value::Table(ref mut table) => {
            let mut rng = rand::thread_rng();
            let output = Rc::make_mut(table);
            output.rows_mut().shuffle(&mut rng);
        }
        _ => {
            return Err(ShellErrorKind::Basic(
                "TypeError",
                format!("Cannot shuffle {}", input.to_type()),
            ))
        }
    }

    ctx.output.push(input)?;

    Ok(())
}
