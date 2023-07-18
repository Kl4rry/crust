use std::rc::Rc;

use once_cell::sync::Lazy;
use rand::prelude::*;

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

static APP: Lazy<App> = Lazy::new(|| App::new("shuffle").about("Shuffle items in container"));

pub fn shuffle(
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

    let mut input = input.unpack();

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
        _ => {
            return Err(ShellErrorKind::Basic(
                "TypeError",
                format!("Cannot shuffle `{}`", input.to_type()),
            ))
        }
    }

    output.push(input);

    Ok(())
}
