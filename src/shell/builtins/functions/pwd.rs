use std::lazy::SyncLazy;

use crate::{
    argparse::{App, ParseErrorKind},
    parser::shell_error::ShellErrorKind,
    shell::{
        stream::{OutputStream, ValueStream},
        Shell, Value,
    },
};

static APP: SyncLazy<App> =
    SyncLazy::new(|| App::new("pwd").about("Print current working directory"));

pub fn pwd(
    _: &mut Shell,
    args: Vec<Value>,
    _: ValueStream,
) -> Result<OutputStream, ShellErrorKind> {
    let _ = match APP.parse(args.into_iter()) {
        Ok(m) => m,
        Err(e) => match e.error {
            ParseErrorKind::Help(m) => return Ok(OutputStream::from_value(Value::String(m))),
            _ => return Err(e.into()),
        },
    };

    let output = OutputStream::from_value(Value::String(String::from(
        std::env::current_dir().unwrap().to_str().unwrap(),
    )));

    Ok(output)
}
