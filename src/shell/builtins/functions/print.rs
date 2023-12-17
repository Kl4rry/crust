use std::io::Write;

use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Arg, Flag, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::value::{SpannedValue, Type},
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("print")
        .about("Print to standard output or standard error")
        .flag(
            Flag::new("stderr")
                .short('e')
                .long("stderr")
                .help("Print to stderr instead"),
        )
        .flag(
            Flag::new("no-newline")
                .short('n')
                .long("no-newline")
                .help("Print a new line between arguments"),
        )
        .arg(
            Arg::new("ARGS", Type::ANY)
                .multiple(true)
                .help("The args that will printed"),
        )
});

pub fn print(ctx: &mut Context, args: Vec<SpannedValue>) -> Result<(), ShellErrorKind> {
    let mut matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let mut output: Box<dyn Write> = if matches.conatins("stderr") {
        Box::new(std::io::stderr())
    } else {
        Box::new(std::io::stdout())
    };

    let no_newline = matches.conatins("no-newline");

    let args = matches.take_values("ARGS");
    if let Some(args) = args {
        for arg in args {
            write!(output, "{}", arg.value).map_err(|e| ShellErrorKind::Io(None, e))?;
            if !no_newline {
                writeln!(output).map_err(|e| ShellErrorKind::Io(None, e))?;
            }
        }
    }
    output.flush().map_err(|e| ShellErrorKind::Io(None, e))?;

    Ok(())
}
