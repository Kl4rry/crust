use std::rc::Rc;

use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Arg, ParseResult},
    parser::shell_error::ShellErrorKind,
    shell::{
        stream::{OutputStream, ValueStream},
        value::{Type, Value},
        Shell,
    },
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("alias")
        .about("Set alias")
        .arg(
            Arg::new("NAME", Type::STRING)
                .help("Name of the alias")
                .required(true),
        )
        .arg(
            Arg::new("COMMAND", Type::STRING)
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
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let name = matches.take_value("NAME").unwrap().unwrap_string();
    let command = matches.take_value("COMMAND").unwrap().unwrap_string();

    if name.is_empty() {
        return Err(ShellErrorKind::Basic(
            "Alias Error",
            format!(
                "alias [NAME] must be atleast one character long\n\n{}",
                APP.usage()
            ),
        ));
    }

    if command.is_empty() {
        return Err(ShellErrorKind::Basic(
            "Alias Error",
            format!(
                "alias [COMMAND] must be atleast one character long\n\n{}",
                APP.usage()
            ),
        ));
    }

    // TODO this should reject anything with a word break char in
    if name.chars().any(|c| c.is_whitespace()) {
        return Err(ShellErrorKind::Basic(
            "Alias Error",
            format!("alias [NAME] cannot contain whitespace\n\n{}", APP.usage()),
        ));
    }

    shell
        .aliases
        .insert(Rc::unwrap_or_clone(name), Rc::unwrap_or_clone(command));
    Ok(())
}
