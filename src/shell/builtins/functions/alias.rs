use std::rc::Rc;

use indexmap::IndexMap;
use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Arg, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::{
        stream::ValueStream,
        value::{table::Table, SpannedValue, Type, Value},
    },
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("alias")
        .about("Set alias")
        .arg(Arg::new("NAME", Type::STRING).help("Name of the alias"))
        .arg(Arg::new("COMMAND", Type::STRING).help("The command that will be run"))
});

pub fn alias(
    ctx: &mut Context,
    args: Vec<SpannedValue>,
    _: ValueStream,
) -> Result<(), ShellErrorKind> {
    let mut matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let name = matches.take_value("NAME").map(|s| s.value.unwrap_string());
    let command = matches
        .take_value("COMMAND")
        .map(|s| s.value.unwrap_string());

    if let Some(name) = name {
        if name.is_empty() {
            return Err(ShellErrorKind::Basic(
                "Alias Error",
                format!(
                    "alias [NAME] must be atleast one character long\n\n{}",
                    APP.usage()
                ),
            ));
        }

        if let Some(command) = command {
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

            ctx.shell
                .aliases
                .insert(Rc::unwrap_or_clone(name), Rc::unwrap_or_clone(command));
        } else if let Some(command) = ctx.shell.aliases.get(&*name) {
            ctx.output.push(Value::from(command.clone()));
        } else {
            return Err(ShellErrorKind::Basic(
                "Alias Error",
                format!("Unknown alias: `{name}`\n\n{}", APP.usage()),
            ));
        }
    } else {
        let mut table = Table::new();
        let alias_header: Rc<str> = Rc::from("alias");
        let command_header: Rc<str> = Rc::from("command");
        for (alias, command) in &ctx.shell.aliases {
            table.insert_map(IndexMap::from([
                (alias_header.clone(), Value::from(alias.clone())),
                (command_header.clone(), Value::from(command.clone())),
            ]));
        }
        ctx.output.push(table.into());
    }

    Ok(())
}
