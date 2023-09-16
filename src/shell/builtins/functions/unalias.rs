use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Arg, ParseResult},
    parser::shell_error::ShellErrorKind,
    shell::{
        frame::Frame,
        stream::{OutputStream, ValueStream},
        value::{SpannedValue, Type},
        Shell,
    },
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("unalias").about("Remove alias").arg(
        Arg::new("NAME", Type::STRING)
            .help("Name of the alias")
            .required(true),
    )
});

pub fn unalias(
    shell: &mut Shell,
    _: &mut Frame,
    args: Vec<SpannedValue>,
    _: ValueStream,
    output: &mut OutputStream,
) -> Result<(), ShellErrorKind> {
    let mut matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let name = matches.take_value("NAME").unwrap().value.unwrap_string();
    if shell.aliases.remove(&*name).is_none() {
        return Err(ShellErrorKind::Basic(
            "Alias Error",
            format!("alias not found\n\n{}", APP.usage()),
        ));
    }

    Ok(())
}
