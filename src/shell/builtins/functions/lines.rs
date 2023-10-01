use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Flag, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::value::{SpannedValue, Value},
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("lines").about("Split a string into lines").flag(
        Flag::new("SKIP")
            .long("skip-empty")
            .short('s')
            .help("Skip empty lines"),
    )
});

pub fn lines(ctx: &mut Context, args: Vec<SpannedValue>) -> Result<(), ShellErrorKind> {
    let matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let skip = matches.conatins("SKIP");

    let mut input = ctx.input.take().unpack();

    match input {
        Value::String(ref mut string) => {
            for line in string.lines() {
                if skip && line.is_empty() {
                    continue;
                }
                ctx.output.push(String::from(line).into());
            }
        }
        _ => {
            return Err(ShellErrorKind::Basic(
                "TypeError",
                format!("Cannot split {} into lines", input.to_type()),
            ))
        }
    }

    Ok(())
}
