use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Flag, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::{current_dir_str, stream::ValueStream, value::SpannedValue, Value},
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("pwd")
        .about("Print current working directory")
        .flag(Flag::new("PHYSICAL").short('p').help("Resolve symlinks"))
});

pub fn pwd(
    ctx: &mut Context,
    args: Vec<SpannedValue>,
    _: ValueStream,
) -> Result<(), ShellErrorKind> {
    let matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    if matches.conatins("PHYSICAL") {
        if let Ok(path) = std::fs::read_link(current_dir_str()) {
            ctx.output
                .push(Value::from(path.to_string_lossy().to_string()));
            return Ok(());
        }
    }

    ctx.output.push(Value::from(current_dir_str()));
    Ok(())
}
