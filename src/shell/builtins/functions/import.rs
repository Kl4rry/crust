use std::rc::Rc;

use once_cell::sync::Lazy;

use super::read_file;
use crate::{
    argparse::{App, Arg, ParseErrorKind},
    parser::shell_error::ShellErrorKind,
    shell::{
        stream::{OutputStream, ValueStream},
        value::{Type, Value},
        Shell,
    },
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("import")
        .about("Import file for http url or filepath")
        .arg(
            Arg::new("path", Type::STRING)
                .help("Path or url to import from")
                .required(true),
        )
});

pub fn import(
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

    let path = matches
        .take_value(&String::from("path"))
        .unwrap()
        .unwrap_string();

    let src = if path.starts_with("https://") || path.starts_with("http://") {
        get_from_url(&path)?
    } else {
        read_file(&*path)?
    };

    shell.run_src(src, Rc::unwrap_or_clone(path), output);
    Ok(())
}

fn get_from_url(path: &str) -> Result<String, ShellErrorKind> {
    let res = ureq::builder().redirects(10).build().get(path).call()?;
    res.into_string().map_err(|e| ShellErrorKind::Io(None, e))
}
