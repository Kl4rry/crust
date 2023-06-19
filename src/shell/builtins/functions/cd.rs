use std::rc::Rc;

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
    App::new("cd")
        .about("Change working directory")
        .arg(Arg::new("DIRECTORY", Type::STRING).help("The new working directory"))
});

pub fn cd(
    shell: &mut Shell,
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

    let dir = match matches.take_value("DIRECTORY") {
        Some(value) => value.unwrap_string(),
        None => Rc::new(shell.home_dir().to_string_lossy().to_string()),
    };

    shell.dir_history.change_dir(&*dir)
}
