use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Arg, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::value::{SpannedValue, Type},
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("unalias").about("Remove alias").arg(
        Arg::new("NAME", Type::STRING)
            .help("Name of the alias")
            .required(true),
    )
});

pub fn unalias(ctx: &mut Context, args: Vec<SpannedValue>) -> Result<(), ShellErrorKind> {
    let mut matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info)?;
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let name = matches.take_value("NAME").unwrap().value.unwrap_string();
    if ctx.shell.aliases.remove(&*name).is_none() {
        return Err(ShellErrorKind::Basic(
            "Alias Error",
            format!("alias not found\n\n{}", APP.usage()),
        ));
    }

    Ok(())
}
