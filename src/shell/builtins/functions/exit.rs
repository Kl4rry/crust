use std::{lazy::SyncLazy, rc::Rc};

use crate::{
    argparse::{App, Arg, ParseErrorKind},
    parser::shell_error::ShellErrorKind,
    shell::{
        stream::{OutputStream, ValueStream},
        value::{Type, Value},
        Shell,
    },
};

static APP: SyncLazy<App> = SyncLazy::new(|| {
    App::new("exit")
        .about("Exit the shell")
        .arg(Arg::new("status", Type::INT).help("The exit status of the shell"))
});

pub fn exit(
    shell: &mut Shell,
    args: Vec<Value>,
    _: ValueStream,
    output: &mut OutputStream,
) -> Result<(), ShellErrorKind> {
    let matches = match APP.parse(args.into_iter()) {
        Ok(m) => m,
        Err(e) => match e.error {
            ParseErrorKind::Help(m) => {
                output.push(Value::String(Rc::new(m)));
                return Ok(());
            }
            _ => return Err(e.into()),
        },
    };

    let status = matches
        .value(&String::from("status"))
        .unwrap_or(&Value::Int(0));
    shell.exit_status = status.unwrap_int();

    shell.running = false;
    Err(ShellErrorKind::Exit)
}
