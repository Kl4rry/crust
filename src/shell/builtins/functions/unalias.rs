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
    App::new("unalias").about("Remove alias").arg(
        Arg::new("name", Type::STRING)
            .help("Name of the alias")
            .required(true),
    )
});

pub fn unalias(
    shell: &mut Shell,
    args: Vec<Value>,
    _: ValueStream,
    output: &mut OutputStream,
) -> Result<(), ShellErrorKind> {
    let mut matches = match APP.parse(args.into_iter()) {
        Ok(m) => m,
        Err(e) => match e.error {
            ParseErrorKind::Help(m) => {
                output.push(m);
                return Ok(());
            }
            _ => return Err(e.into()),
        },
    };

    let name = matches
        .take_value(&String::from("name"))
        .unwrap()
        .unwrap_string();

    let r = shell.aliases.remove(&*name);
    if r.is_none() {
        return Err(ShellErrorKind::Basic(
            "Alias Error",
            format!("alias not found\n\n{}", APP.usage()),
        ));
    }

    Ok(())
}
