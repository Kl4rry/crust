use std::io::{stdout, Write};

use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Flag, ParseResult},
    parser::shell_error::ShellErrorKind,
    shell::{
        stream::{OutputStream, ValueStream},
        value::Value,
        Shell,
    },
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("clear").about("Clears the terminal").flag(
        Flag::new("SCROLLBACK")
            .short('x')
            .help("Do not clear scrollback"),
    )
});

pub fn clear(
    _: &mut Shell,
    args: Vec<Value>,
    _: ValueStream,
    output: &mut OutputStream,
) -> Result<(), ShellErrorKind> {
    let matches = match APP.parse(args.into_iter()) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    if matches.get("SCROLLBACK").is_some() {
        write!(
            stdout(),
            "{}{}",
            ansi_escapes::EraseScreen,
            ansi_escapes::CursorTo::TopLeft
        )
        .map_err(|err| ShellErrorKind::Io(None, err))?;
    } else {
        //https://superuser.com/questions/1628694/how-do-i-add-a-keyboard-shortcut-to-clear-scrollback-buffer-in-windows-terminal
        write!(stdout(), "{}", ansi_escapes::ClearScreen)
            .map_err(|err| ShellErrorKind::Io(None, err))?;
    }
    stdout()
        .flush()
        .map_err(|err| ShellErrorKind::Io(None, err))?;

    Ok(())
}
