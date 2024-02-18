use std::{path::PathBuf, rc::Rc};

use num_traits::ToBytes;
use once_cell::sync::Lazy;

use super::save_file;
use crate::{
    argparse::{App, Arg, Flag, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::value::{save::save_value, SpannedValue, Type, Value},
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
                .help("Save raw text data")
                .conflicts_with("RAW".into()),
        )
        .flag(
            Flag::new("RAW")
                .long("raw")
                .short('r')
                .help("Save raw binary data")
                .conflicts_with("STR".into()),
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

pub fn save(ctx: &mut Context, args: Vec<SpannedValue>) -> Result<(), ShellErrorKind> {
    let mut matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let path = PathBuf::from(
        matches
            .take_value(&String::from("PATH"))
            .unwrap()
            .value
            .unwrap_string()
            .as_str(),
    );

    let pretty = matches.conatins(&String::from("PRETTY"));
    let append = matches.conatins(&String::from("APPEND"));

    let input = ctx.input.take();

    if matches.conatins("STR") {
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
                    format!("Cannot serialize {t} to string"),
                ))
            }
        };
        save_file(path, data.as_bytes(), append)?;
    } else if matches.conatins("RAW") {
        let value: Value = input.unpack();
        let t = value.to_type();
        let data = match value {
            Value::String(mut string) => {
                Rc::make_mut(&mut string);
                Rc::into_inner(string).unwrap().into_bytes()
            }
            Value::Int(ref int) => Vec::from(int.to_ne_bytes()),
            Value::Float(ref float) => Vec::from(float.to_ne_bytes()),
            Value::Bool(ref boolean) => {
                if *boolean {
                    vec![1]
                } else {
                    vec![0]
                }
            }
            Value::Binary(mut data) => {
                Rc::make_mut(&mut data);
                Rc::into_inner(data).unwrap()
            }
            _ => {
                return Err(ShellErrorKind::Basic(
                    "Serialization Error",
                    format!("Cannot write {t} as binary data"),
                ))
            }
        };
        save_file(path, &data, append)?;
    } else {
        save_value(path, input, append, pretty)?;
    }

    Ok(())
}
