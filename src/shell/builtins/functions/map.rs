use std::{iter, rc::Rc};

use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Arg, Flag, ParseResult},
    parser::{
        ast::{context::Context, expr::closure::Closure},
        shell_error::ShellErrorKind,
    },
    shell::{
        frame::Frame,
        stream::OutputStream,
        value::{SpannedValue, Type, Value},
    },
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("map")
        .arg(
            Arg::new("CLOSURE", Type::CLOSURE)
                .required(true)
                .help("Closure to apply on the items"),
        )
        .flag(
            Flag::new("EMPTY")
                .long("empty")
                .short('e')
                .help("Keep empty items"),
        )
        .about("Apply closure to a sequence")
});

pub fn map(ctx: &mut Context, args: Vec<SpannedValue>) -> Result<(), ShellErrorKind> {
    let mut matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let empty = matches.conatins("EMPTY");
    let closure = matches
        .take_value("CLOSURE")
        .unwrap()
        .value
        .unwrap_closure();

    let value = match ctx.input.take().unpack() {
        Value::Null => Value::Null,
        Value::Int(int) => apply_closure(ctx, closure, iter::once(int.into()), empty)?,
        Value::Float(float) => apply_closure(ctx, closure, iter::once(float.into()), empty)?,
        Value::Bool(boolean) => apply_closure(ctx, closure, iter::once(boolean.into()), empty)?,
        Value::String(string) => {
            apply_closure(ctx, closure, string.chars().map(Value::from), empty)?
        }
        Value::List(list) => {
            apply_closure(ctx, closure, Rc::unwrap_or_clone(list).into_iter(), empty)?
        }
        Value::Map(map) => apply_closure(
            ctx,
            closure,
            Rc::unwrap_or_clone(map)
                .into_iter()
                .map(|(key, value)| Value::from(vec![Value::from(key.to_string()), value])),
            empty,
        )?,
        Value::Table(table) => apply_closure(
            ctx,
            closure,
            Rc::unwrap_or_clone(table)
                .iter()
                .map(|row| Value::Map(row.into())),
            empty,
        )?,
        Value::Range(range) => {
            apply_closure(ctx, closure, (*range).clone().map(Value::from), empty)?
        }
        Value::Regex(regex) => apply_closure(ctx, closure, iter::once(Value::Regex(regex)), empty)?,
        Value::Binary(binary) => apply_closure(
            ctx,
            closure,
            binary.iter().copied().map(|b| Value::from(b as i64)),
            empty,
        )?,
        Value::Closure(c) => apply_closure(ctx, closure, iter::once(Value::Closure(c)), empty)?,
    };

    ctx.output.push(value);
    Ok(())
}

fn apply_closure(
    ctx: &mut Context,
    closure: Rc<(Rc<Closure>, Frame)>,
    iter: impl Iterator<Item = Value>,
    keep_empty: bool,
) -> Result<Value, ShellErrorKind> {
    let (closure, frame) = &*closure;
    let mut output = Vec::new();
    for value in iter {
        let mut capture = OutputStream::new_capture();
        let mut ctx = Context {
            shell: ctx.shell,
            frame: frame.clone(),
            output: &mut capture,
            input: ctx.input,
            src: ctx.src.clone(),
        };
        closure.eval(&mut ctx, iter::once(value))?;
        let item = capture.into_value_stream().unpack();
        if keep_empty || item != Value::Null {
            output.push(item);
        }
    }
    Ok(output.into())
}
