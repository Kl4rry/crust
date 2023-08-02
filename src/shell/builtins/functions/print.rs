use std::io::Write;

use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Arg, Flag, ParseResult},
    parser::shell_error::ShellErrorKind,
    shell::{
        frame::Frame,
        stream::{OutputStream, ValueStream},
        value::{SpannedValue, Type},
        Shell,
    },
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("cd")
        .about("Change working directory")
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

pub fn print(
    _: &mut Shell,
    _: &mut Frame,
    args: Vec<SpannedValue>,
    _: ValueStream,
    output: &mut OutputStream,
) -> Result<(), ShellErrorKind> {
    let mut matches = match APP.parse(args.into_iter().map(|v| v.into())) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            output.push(info);
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
            write!(output, "{}", arg).map_err(|e| ShellErrorKind::Io(None, e))?;
            if !no_newline {
                writeln!(output).map_err(|e| ShellErrorKind::Io(None, e))?;
            }
        }
    }
    output.flush().map_err(|e| ShellErrorKind::Io(None, e))?;

    Ok(())
}
