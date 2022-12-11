use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Flag, ParseResult},
    parser::shell_error::ShellErrorKind,
    shell::{
        current_dir_str,
        frame::Frame,
        stream::{OutputStream, ValueStream},
        value::SpannedValue,
        Shell, Value,
    },
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("pwd")
        .about("Print current working directory")
        .flag(Flag::new("PHYSICAL").short('p').help("Resolve symlinks"))
});

pub fn pwd(
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

    if matches.conatins("PHYSICAL") {
        if let Ok(path) = std::fs::read_link(current_dir_str()) {
            output.push(Value::from(path.to_string_lossy().to_string()));
            return Ok(());
        }
    }

    output.push(Value::from(current_dir_str()));
    Ok(())
}
