use std::io::{self, Write};

use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Arg, ParseResult},
    parser::shell_error::ShellErrorKind,
    shell::{
        frame::Frame,
        stream::{OutputStream, ValueStream},
        value::{SpannedValue, Type},
        Shell,
    },
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("input")
        .about("Get input from stdin")
        .arg(Arg::new("PROMPT", Type::STRING).help("Prompt to print before input"))
});

pub fn input(
    _: &mut Shell,
    _: &mut Frame,
    args: Vec<SpannedValue>,
    _: ValueStream,
    output: &mut OutputStream,
) -> Result<(), ShellErrorKind> {
    let matches = match APP.parse(args.into_iter().map(|v| v.into())) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    if let Some(prompt) = matches.value("PROMPT") {
        print!("{}", prompt.unwrap_as_str());
        io::stdout()
            .flush()
            .map_err(|err| ShellErrorKind::Io(None, err))?;
    }

    // FIXME This cannot be broken by CTRL + C currently
    let mut buffer = String::new();
    io::stdin()
        .read_line(&mut buffer)
        .map_err(|err| ShellErrorKind::Io(None, err))?;

    output.push(buffer.into());
    Ok(())
}
