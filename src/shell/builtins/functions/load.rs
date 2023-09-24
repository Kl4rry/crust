use std::{path::PathBuf, rc::Rc};

use once_cell::sync::Lazy;

use super::{read_file, read_file_raw};
use crate::{
    argparse::{App, Arg, Flag, ParseResult},
    parser::shell_error::ShellErrorKind,
    shell::{
        frame::Frame,
        stream::{OutputStream, ValueStream},
        value::{SpannedValue, Type, Value},
        Shell,
    },
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("load")
        .about("Load a data from file")
        .arg(
            Arg::new("PATH", Type::STRING)
                .help("Path to load data from")
                .required(true),
        )
        .flag(
            Flag::new("STR")
                .long("str")
                .short('s')
                .help("Load raw text data")
                .conflicts_with("RAW".into()),
        )
        .flag(
            Flag::new("RAW")
                .long("raw")
                .short('r')
                .help("Load raw binary data")
                .conflicts_with("STR".into()),
        )
});

pub fn load(
    _: &mut Shell,
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

    let path = PathBuf::from(
        matches
            .take_value("PATH")
            .unwrap()
            .value
            .unwrap_string()
            .as_str(),
    );

    if matches.conatins("STR") {
        let file = read_file(&path)?;
        output.push(Value::String(Rc::new(file)));
    } else if matches.conatins("RAW") {
        let file = read_file_raw(&path)?;
        output.push(Value::Binary(Rc::new(file)));
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
                "txt" => {
                    let file = read_file(&path)?;
                    output.push(file.into());
                }
                _ => return Err(ShellErrorKind::UnknownFileType(ext)),
            }
        } else {
            let file = read_file_raw(&path)?;
            match String::from_utf8(file) {
                Ok(string) => output.push(string.into()),
                Err(e) => output.push(e.into_bytes().into()),
            }
        }
    }

    Ok(())
}
