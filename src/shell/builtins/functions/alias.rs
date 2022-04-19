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
    App::new("alias")
        .about("Set alias")
        .arg(
            Arg::new("name", Type::STRING)
                .help("Name of the alias")
                .required(true),
        )
        .arg(
            Arg::new("command", Type::STRING)
                .help("The command that will be run")
                .required(true),
        )
});

pub fn alias(
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
    let command = matches
        .take_value(&String::from("command"))
        .unwrap()
        .unwrap_string();

    if name.is_empty() {
        return Err(ShellErrorKind::Basic(
            "Alias Error",
            format!(
                "alias [name] must be atleast one character long\n\n{}",
                APP.usage()
            ),
        ));
    }

    if command.is_empty() {
        return Err(ShellErrorKind::Basic(
            "Alias Error",
            format!(
                "alias [command] must be atleast one character long\n\n{}",
                APP.usage()
            ),
        ));
    }

    if name.chars().any(|c| c.is_whitespace()) {
        return Err(ShellErrorKind::Basic(
            "Alias Error",
            format!("alias [name] cannot contain whitespace\n\n{}", APP.usage()),
        ));
    }

    shell
        .aliases
        .insert(Rc::unwrap_or_clone(name), Rc::unwrap_or_clone(command));
    Ok(())
}
