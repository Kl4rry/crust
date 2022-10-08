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
    App::new("open")
        .about("Open a file or url with the default program")
        .arg(
            Arg::new("PATH", Type::STRING)
                .help("Path to the file or directory to open")
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
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let path = matches
        .take_value(&String::from("PATH"))
        .unwrap()
        .unwrap_string();

    opener::open(&*path)?;
    Ok(())
}
