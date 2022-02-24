use std::{lazy::SyncLazy, path::Path};

use crate::{
    argparse::{App, Arg, ParseErrorKind},
    parser::shell_error::ShellErrorKind,
    shell::{
        stream::{OutputStream, ValueStream},
        value::{Type, Value},
        Shell,
    },
};

static APP: SyncLazy<App> = SyncLazy::new(|| {
    App::new("cd")
        .about("Change working directory")
        .arg(Arg::new("directory", Type::String).help("The new working directory"))
});

pub fn cd(
    shell: &mut Shell,
    args: Vec<Value>,
    _: ValueStream,
    output: &mut OutputStream,
) -> Result<(), ShellErrorKind> {
    let matches = match APP.parse(args.into_iter()) {
        Ok(m) => m,
        Err(e) => match e.error {
            ParseErrorKind::Help(m) => {
                output.push(Value::String(m));
                return Ok(());
            }
            _ => return Err(e.into()),
        },
    };

    let temp;
    let dir = match matches.value(&String::from("directory")) {
        Some(value) => match value {
            Value::String(ref s) => s.as_str(),
            _ => panic!("directory must be string this is a bug"),
        },
        None => {
            temp = shell.home_dir.to_string_lossy();
            &temp
        }
    };

    let new_dir = Path::new(dir);
    std::env::set_current_dir(&new_dir).map_err(|err| ShellErrorKind::Io(None, err))?;
    Ok(())
}
