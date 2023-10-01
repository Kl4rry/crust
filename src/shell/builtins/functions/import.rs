use std::rc::Rc;

use once_cell::sync::Lazy;

use super::read_file;
use crate::{
    argparse::{App, Arg, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::{
        stream::ValueStream,
        value::{SpannedValue, Type},
    },
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("import")
        .about("Import file for http url or filepath")
        .arg(
            Arg::new("URL", Type::STRING)
                .help("Path or url to import from")
                .required(true),
        )
});

pub fn import(ctx: &mut Context, args: Vec<SpannedValue>) -> Result<(), ShellErrorKind> {
    let mut matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let path = matches
        .take_value(&String::from("URL"))
        .unwrap()
        .value
        .unwrap_string();

    let src = if path.starts_with("https://") || path.starts_with("http://") {
        get_from_url(&path)?
    } else {
        read_file(&*path)?
    };

    ctx.shell.run_src(
        Rc::unwrap_or_clone(path),
        src,
        ctx.output,
        ValueStream::new(),
    );
    Ok(())
}

fn get_from_url(path: &str) -> Result<String, ShellErrorKind> {
    let res = ureq::builder().redirects(10).build().get(path).call()?;
    res.into_string().map_err(|e| ShellErrorKind::Io(None, e))
}
