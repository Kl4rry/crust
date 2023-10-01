use std::rc::Rc;

use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Arg, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::value::{SpannedValue, Type},
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("cd")
        .about("Change working directory")
        .arg(Arg::new("DIRECTORY", Type::STRING).help("The new working directory"))
});

pub fn cd(ctx: &mut Context, args: Vec<SpannedValue>) -> Result<(), ShellErrorKind> {
    let mut matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let dir = match matches.take_value("DIRECTORY") {
        Some(value) => value.value.unwrap_string(),
        None => Rc::new(ctx.shell.home_dir().to_string_lossy().to_string()),
    };

    if dir.as_str() == "-" {
        ctx.shell.dir_history.back()
    } else {
        ctx.shell.dir_history.change_dir(&*dir)
    }
}
