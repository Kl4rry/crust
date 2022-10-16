use once_cell::sync::Lazy;

use crate::{
    argparse::{App, ParseResult},
    parser::shell_error::ShellErrorKind,
    shell::{
        current_dir_str,
        frame::Frame,
        stream::{OutputStream, ValueStream},
        Shell, Value,
    },
};

static APP: Lazy<App> = Lazy::new(|| App::new("pwd").about("Print current working directory"));

pub fn pwd(
    _: &mut Shell,
    _: &mut Frame,
    args: Vec<Value>,
    _: ValueStream,
    output: &mut OutputStream,
) -> Result<(), ShellErrorKind> {
    let _ = match APP.parse(args.into_iter()) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    output.push(Value::from(current_dir_str()));

    Ok(())
}
