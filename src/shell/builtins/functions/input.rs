use std::io::{self, Write};

use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Arg, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::{
        stream::ValueStream,
        value::{SpannedValue, Type},
    },
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("input")
        .about("Get input from stdin")
        .arg(Arg::new("PROMPT", Type::STRING).help("Prompt to print before input"))
});

pub fn input(
    ctx: &mut Context,
    args: Vec<SpannedValue>,
    _: ValueStream,
) -> Result<(), ShellErrorKind> {
    let matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    if let Some(prompt) = matches.value("PROMPT") {
        print!("{}", prompt.value.unwrap_as_str());
        io::stdout()
            .flush()
            .map_err(|err| ShellErrorKind::Io(None, err))?;
    }

    // FIXME This cannot be broken by CTRL + C currently
    let mut buffer = String::new();
    io::stdin()
        .read_line(&mut buffer)
        .map_err(|err| ShellErrorKind::Io(None, err))?;

    ctx.output.push(buffer.into());
    Ok(())
}
