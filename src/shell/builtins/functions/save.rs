use std::path::PathBuf;

use once_cell::sync::Lazy;

use super::save_file;
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
    App::new("save")
        .about("Save data to file")
        .arg(
            Arg::new("PATH", Type::STRING)
                .help("File path")
                .required(true),
        )
        .flag(
            Flag::new("STR")
                .long("str")
                .short('s')
                .help("Save raw text data"),
        )
        .flag(
            Flag::new("PRETTY")
                .long("pretty")
                .short('p')
                .help("Prettify the saved data"),
        )
        .flag(
            Flag::new("APPEND")
                .long("append")
                .short('a')
                .help("Append data to the end of the file"),
        )
});

pub fn save(
    _: &mut Shell,
    _: &mut Frame,
    args: Vec<SpannedValue>,
    input: ValueStream,
    output: &mut OutputStream,
) -> Result<(), ShellErrorKind> {
    let mut matches = match APP.parse(args.into_iter().map(|v| v.into())) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let path = PathBuf::from(
        matches
            .take_value(&String::from("PATH"))
            .unwrap()
            .unwrap_string()
            .as_str(),
    );

    let pretty = matches.conatins(&String::from("PRETTY"));
    let append = matches.conatins(&String::from("APPEND"));

    if matches.conatins(&String::from("STR")) {
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
                "txt" => {
                    // TODO handle txt
                    todo!()
                }
                _ => return Err(ShellErrorKind::UnknownFileType(ext)),
            }
        }
    }

    Ok(())
}
