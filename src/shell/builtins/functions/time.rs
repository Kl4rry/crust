use std::{iter, time::Instant};

use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Arg, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::value::{SpannedValue, Type, Value},
};

// TODO add output formats
static APP: Lazy<App> = Lazy::new(|| {
    App::new("time")
        .about("Measure time it takes to execute closure")
        .arg(
            Arg::new("CLOSURE", Type::CLOSURE)
                .required(true)
                .help("The closure to be executed"),
        )
});

pub fn time(ctx: &mut Context, args: Vec<SpannedValue>) -> Result<(), ShellErrorKind> {
    let mut matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info)?;
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let (closure, frame) = &*matches
        .take_value("CLOSURE")
        .unwrap()
        .value
        .unwrap_closure();

    let mut ctx = Context {
        shell: ctx.shell,
        frame: frame.clone(),
        output: ctx.output,
        input: ctx.input,
        src: ctx.src.clone(),
    };
    let before = Instant::now();
    closure.eval(&mut ctx, iter::empty())?;
    let duration = Instant::now().duration_since(before);
    ctx.output.push(Value::from(format!("{:?}", duration)))?;

    Ok(())
}
