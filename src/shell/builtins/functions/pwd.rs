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
    output: &mut OutputStream,
) -> Result<(), ShellErrorKind> {
    let _ = match APP.parse(args.into_iter()) {
        Ok(m) => m,
        Err(e) => match e.error {
            ParseErrorKind::Help(m) => {
                output.push(Value::String(m));
                return Ok(());
            }
            _ => return Err(e.into()),
        },
    };

    output.push(Value::String(
        std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .to_string(),
    ));

    Ok(())
}
