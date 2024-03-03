use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Arg, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::value::{SpannedValue, Type},
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("glob")
        .about("Match against files or folders with pattern")
        .arg(
            Arg::new("PATTERN", Type::STRING)
                .multiple(true)
                .required(true)
                .help("Pattern to match against"),
        )
});

pub fn glob(ctx: &mut Context, args: Vec<SpannedValue>) -> Result<(), ShellErrorKind> {
    let mut matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info)?;
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let pattern = matches.take_value("PATTERN").unwrap().value.unwrap_string();

    for path in glob::glob(&pattern)? {
        let path = path?;
        let string = path.to_string_lossy();
        ctx.output.push((&*string).into())?;
    }

    Ok(())
}
