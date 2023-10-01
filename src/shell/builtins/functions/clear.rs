use std::io::{stdout, Write};

use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Flag, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::value::SpannedValue,
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("clear").about("Clears the terminal").flag(
        Flag::new("SCROLLBACK")
            .short('x')
            .help("Do not clear scrollback"),
    )
});

pub fn clear(ctx: &mut Context, args: Vec<SpannedValue>) -> Result<(), ShellErrorKind> {
    let matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    const ERASE_SCREEN: &str = "\x1B[2J";
    const CURSOR_TO_TOPLEFT: &str = "\x1B[1;1H";
    const CLEAR_SCREEN: &str = "\u{001b}c";

    if matches.get("SCROLLBACK").is_some() {
        write!(stdout(), "{}{}", ERASE_SCREEN, CURSOR_TO_TOPLEFT)
            .map_err(|err| ShellErrorKind::Io(None, err))?;
    } else {
        //https://superuser.com/questions/1628694/how-do-i-add-a-keyboard-shortcut-to-clear-scrollback-buffer-in-windows-terminal
        write!(stdout(), "{}", CLEAR_SCREEN).map_err(|err| ShellErrorKind::Io(None, err))?;
    }
    stdout()
        .flush()
        .map_err(|err| ShellErrorKind::Io(None, err))?;

    Ok(())
}
