use std::{path::Path, rc::Rc};

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
    App::new("cd")
        .about("Change working directory")
        .arg(Arg::new("directory", Type::STRING).help("The new working directory"))
});

pub fn cd(
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

    let dir = match matches.take_value(&String::from("directory")) {
        Some(value) => value.unwrap_string(),
        None => Rc::new(shell.home_dir().to_string_lossy().to_string()),
    };

    let new_dir = Path::new(&*dir);
    std::env::set_current_dir(new_dir).map_err(|err| ShellErrorKind::Io(None, err))?;
    Ok(())
}
