use once_cell::sync::Lazy;

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
    App::new("open")
        .about("Open a file or url with the default program")
        .arg(
            Arg::new("path", Type::STRING)
                .help("Path or url to import from")
                .required(true),
        )
});

pub fn open(
    _: &mut Shell,
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

    opener::open(&*path)?;
    Ok(())
}
