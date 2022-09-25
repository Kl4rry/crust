use std::path::PathBuf;

use once_cell::sync::Lazy;

use super::save_file;
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
    App::new("save")
        .about("Save data to file")
        .arg(
            Arg::new("path", Type::STRING)
                .help("File path")
                .required(true),
        )
        .flag(
            Flag::new("str")
                .long("str")
                .short('s')
                .help("Save raw text data"),
        )
        .flag(
            Flag::new("pretty")
                .long("pretty")
                .short('p')
                .help("Prettify the saved data"),
        )
        .flag(
            Flag::new("append")
                .long("append")
                .short('a')
                .help("Append data to the end of the file"),
        )
});

pub fn save(
    _: &mut Shell,
    args: Vec<Value>,
    input: ValueStream,
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

    let pretty = matches.conatins(&String::from("pretty"));
    let append = matches.conatins(&String::from("append"));

    if matches.conatins(&String::from("str")) {
        let value: Value = input.unpack();
        let t = value.to_type();
        let data = match value {
            Value::String(string) => (*string).clone(),
            Value::Int(int) => int.to_string(),
            Value::Float(float) => float.to_string(),
            Value::Bool(boolean) => boolean.to_string(),
            _ => {
                return Err(ShellErrorKind::Basic(
                    "Serialization Error",
                    format!("Cannot serialize `{t}` to string"),
                ))
            }
        };
        save_file(path, data.as_bytes(), append)?;
    } else {
        let ext = path.extension();
        if let Some(ext) = ext {
            let ext = ext.to_string_lossy().to_string();
            match ext.as_str() {
                "json" => {
                    let data = if pretty {
                        serde_json::to_string_pretty(&input.unpack())?
                    } else {
                        serde_json::to_string(&input.unpack())?
                    };
                    save_file(path, data.as_bytes(), append)?;
                }
                "toml" => {
                    let data = if pretty {
                        toml::to_string_pretty(&input.unpack())?
                    } else {
                        toml::to_string(&input.unpack())?
                    };
                    save_file(path, data.as_bytes(), append)?;
                }
                _ => return Err(ShellErrorKind::UnknownFileType(ext)),
            }
        }
    }

    Ok(())
}
