use std::lazy::SyncLazy;

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
        .arg(Arg::new("status", Type::Int).help("The exit status of the shell"))
});

pub fn exit(
    shell: &mut Shell,
    args: Vec<Value>,
    _: ValueStream,
) -> Result<OutputStream, ShellErrorKind> {
    let matches = match APP.parse(args.into_iter()) {
        Ok(m) => m,
        Err(e) => match e.error {
            ParseErrorKind::Help(m) => return Ok(OutputStream::from_value(Value::String(m))),
            _ => return Err(e.into()),
        },
    };

    let status = matches
        .value(&String::from("status"))
        .unwrap_or(&Value::Int(0));
    shell.exit_status = status.unwrap_i128();

    shell.running = false;
    Err(ShellErrorKind::Exit)
}
