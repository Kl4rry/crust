use std::{iter, rc::Rc};

use indexmap::IndexMap;
use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Arg, ParseResult},
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
    App::new("filter")
        .arg(
            Arg::new("CLOSURE", Type::CLOSURE)
                .required(true)
                .help("Predicate closure"),
        )
        .about("Filter out sequence using predicate closure")
});

pub fn filter(ctx: &mut Context, args: Vec<SpannedValue>) -> Result<(), ShellErrorKind> {
    let mut matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let closure = matches
        .take_value("CLOSURE")
        .unwrap()
        .value
        .unwrap_closure();

    let value = ctx.input.take().unpack();
    let t = value.to_type();
    let value = match value {
        Value::String(string) => {
            let mut output = String::new();
            for (ch, value) in string.chars().map(|ch| (ch, Value::from(ch))) {
                if apply_closure(ctx, &closure, value)? {
                    output.push(ch);
                }
            }
            Value::from(output)
        }
        Value::List(list) => {
            let mut output = Vec::new();
            for value in list.iter() {
                if apply_closure(ctx, &closure, value.clone())? {
                    output.push(value.clone());
                }
            }
            Value::from(output)
        }
        Value::Map(map) => {
            let mut output = IndexMap::new();
            for (k, v) in map.iter() {
                let keep = apply_closure(
                    ctx,
                    &closure,
                    Value::from(vec![Value::from(k.to_string()), v.clone()]),
                )?;
                if keep {
                    output.insert(k.clone(), v.clone());
                }
            }
            Value::from(output)
        }
        Value::Table(mut table) => {
            let mut keep_rows = Vec::new();
            let headers: Vec<_> = table.headers().iter().collect();
            for row in table.rows() {
                let map: IndexMap<_, _> = headers
                    .iter()
                    .zip(row.iter())
                    .map(|(k, v)| ((*k).clone(), v.clone()))
                    .collect();
                keep_rows.push(apply_closure(ctx, &closure, Value::from(map))?);
            }
            {
                let table = Rc::make_mut(&mut table);
                let rows = table.rows_mut();
                let mut i = 0;
                rows.retain(|_| {
                    let b = keep_rows[i];
                    i += 1;
                    b
                });
            }

            Value::Table(table)
        }
        Value::Range(range) => {
            let mut output = Vec::new();
            for value in (*range).clone().map(Value::from).clone() {
                if apply_closure(ctx, &closure, value.clone())? {
                    output.push(value.clone());
                }
            }
            Value::from(output)
        }
        Value::Binary(binary) => {
            let mut output = Vec::new();
            for value in binary.iter().copied() {
                if apply_closure(ctx, &closure, Value::from(value as i64))? {
                    output.push(value);
                }
            }
            Value::from(output)
        }
        _ => {
            return Err(ShellErrorKind::Basic(
                "TypeError",
                format!("Filter does not support {}", t),
            ))
        }
    };

    ctx.output.push(value);
    Ok(())
}

fn apply_closure(
    ctx: &mut Context,
    closure: &Rc<(Rc<Closure>, Frame)>,
    value: Value,
) -> Result<bool, ShellErrorKind> {
    let (closure, frame) = &**closure;
    let mut capture = OutputStream::new_capture();
    let mut ctx = Context {
        shell: ctx.shell,
        frame: frame.clone(),
        output: &mut capture,
        input: ctx.input,
        src: ctx.src.clone(),
    };
    closure.eval(&mut ctx, iter::once(value))?;
    Ok(capture.into_value_stream().unpack().truthy())
}
