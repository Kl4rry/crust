use std::{lazy::SyncLazy, rc::Rc};

use crate::{
    argparse::{App, Flag, ParseErrorKind},
    parser::shell_error::ShellErrorKind,
    shell::{
        stream::{OutputStream, ValueStream},
        value::Value,
        Shell,
    },
};

static APP: SyncLazy<App> = SyncLazy::new(|| {
    App::new("clear").about("Clears the terminal").flag(
        Flag::new("SCROLLBACK")
            .short('x')
            .help("Do not clear scrollback"),
    )
});

pub fn env(
    shell: &mut Shell,
    args: Vec<Value>,
    _: ValueStream,
    output: &mut OutputStream,
) -> Result<(), ShellErrorKind> {
    let _ = match APP.parse(args.into_iter()) {
        Ok(m) => m,
        Err(e) => match e.error {
            ParseErrorKind::Help(m) => {
                output.push(m);
                return Ok(());
            }
            _ => return Err(e.into()),
        },
    };

    for (key, value) in shell.env() {
        output.push(Value::String(Rc::new(format!("{}={}\n", key, value))));
    }

    Ok(())
}
