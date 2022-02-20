use std::{fs, io, lazy::SyncLazy};

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
    App::new("import")
        .about("Clears the terminal")
        .arg(Arg::new("path", Type::String).required(true))
});

pub fn import(
    shell: &mut Shell,
    args: Vec<Value>,
    _: ValueStream,
) -> Result<OutputStream, ShellErrorKind> {
    let matches = match APP.parse(args.into_iter()) {
        Ok(m) => m,
        Err(e) => match e.error {
            ParseErrorKind::Help(m) => return Ok(OutputStream::from_value(Value::String(m))),
            _ => return Err(e.into()),
        },
    };

    let path = matches.value(&String::from("path")).unwrap().to_string();

    let src = if path.starts_with("https://") || path.starts_with("http://") {
        get_from_url(&path)?
    } else {
        get_from_file(&path)?
    };

    // this bad
    // it does not capture the output stream correctly
    // it should probably be possible to pipe this output into another command
    shell.run_src(src, path);

    Ok(OutputStream::default())
}

fn get_from_url(path: &str) -> Result<String, ShellErrorKind> {
    let res = ureq::builder().redirects(10).build().get(path).call()?;
    res.into_string().map_err(|e| ShellErrorKind::Io(None, e))
}

fn get_from_file(path: &str) -> Result<String, ShellErrorKind> {
    fs::read_to_string(path).map_err(|e| file_err_to_shell_err(e, path.to_string()))
}

fn file_err_to_shell_err(error: io::Error, name: String) -> ShellErrorKind {
    match error.kind() {
        io::ErrorKind::NotFound => ShellErrorKind::FileNotFound(name),
        io::ErrorKind::PermissionDenied => ShellErrorKind::FilePermissionDenied(name),
        _ => ShellErrorKind::Io(None, error),
    }
}
