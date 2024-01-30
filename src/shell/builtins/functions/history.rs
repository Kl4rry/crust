use once_cell::sync::Lazy;
use rustyline::history::History;

use crate::{
    argparse::{App, Flag, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::value::{SpannedValue, Value},
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("history")
        .about("Display or edit history")
        .flag(Flag::new("CLEAR").short('c').long("clear"))
});

pub fn history(ctx: &mut Context, args: Vec<SpannedValue>) -> Result<(), ShellErrorKind> {
    let matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    if matches.conatins("CLEAR") {
        let history = ctx.shell.editor.history_mut();
        let _ = history.clear();
    } else {
        let history = ctx.shell.editor.history();
        let mut output = Vec::new();
        for entry in history {
            output.push(Value::from(entry.as_str()));
        }
        ctx.output.push(output.into());
    }

    Ok(())
}
