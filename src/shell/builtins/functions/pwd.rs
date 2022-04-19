use std::{lazy::SyncLazy, rc::Rc};

use crate::{
    argparse::{App, ParseErrorKind},
    parser::shell_error::ShellErrorKind,
    shell::{
        current_dir_str,
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
                output.push(Value::String(Rc::new(m)));
                return Ok(());
            }
            _ => return Err(e.into()),
        },
    };

    output.push(Value::String(Rc::new(current_dir_str())));

    Ok(())
}
