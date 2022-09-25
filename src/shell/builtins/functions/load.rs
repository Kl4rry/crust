use std::{path::PathBuf, rc::Rc};

use once_cell::sync::Lazy;

use super::read_file;
use crate::{
    argparse::{App, Arg, Flag, ParseErrorKind},
    parser::shell_error::ShellErrorKind,
    shell::{
        stream::{OutputStream, ValueStream},
        value::{Type, Value},
        Shell,
    },
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("load")
        .about("Load a data from file")
        .arg(
            Arg::new("path", Type::STRING)
                .help("Path to load data from")
                .required(true),
        )
        .flag(
            Flag::new("str")
                .long("str")
                .short('s')
                .help("Load raw text data"),
        )
});

pub fn load(
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

    let path = PathBuf::from(
        matches
            .take_value(&String::from("path"))
            .unwrap()
            .unwrap_string()
            .as_str(),
    );

    if matches.conatins(&String::from("str")) {
        let file = read_file(&path)?;
        output.push(Value::String(Rc::new(file)));
    } else {
        let ext = path.extension();
        if let Some(ext) = ext {
            let ext = ext.to_string_lossy().to_string();
            match ext.as_str() {
                "json" => {
                    let file = read_file(&path)?;
                    output.push(serde_json::from_str(&file)?);
                }
                "toml" => {
                    let file = read_file(&path)?;
                    output.push(toml::from_str(&file)?);
                }
                _ => return Err(ShellErrorKind::UnknownFileType(ext)),
            }
        }
    }

    Ok(())
}
