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
    App::new("unalias").about("Remove alias").arg(
        Arg::new("NAME", Type::STRING)
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
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let name = matches.take_value("NAME").unwrap().unwrap_string();

    let r = shell.aliases.remove(&*name);
    if r.is_none() {
        return Err(ShellErrorKind::Basic(
            "Alias Error",
            format!("alias not found\n\n{}", APP.usage()),
        ));
    }

    Ok(())
}
