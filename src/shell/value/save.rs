use std::{borrow::Cow, path::Path};

use super::Value;
use crate::{
    parser::shell_error::ShellErrorKind,
    shell::{builtins::functions::save_file, stream::ValueStream},
};

pub fn save_value(
    path: impl AsRef<Path>,
    input: ValueStream,
    append: bool,
    pretty: bool,
) -> Result<(), ShellErrorKind> {
    let path = path.as_ref();
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
                save_file(path, data.as_bytes(), append)
            }
            "toml" => {
                let data = if pretty {
                    toml::to_string_pretty(&input.unpack())?
                } else {
                    toml::to_string(&input.unpack())?
                };
                save_file(path, data.as_bytes(), append)
            }
            _ => {
                // TODO use try_expand_to_strings
                let input = input.unpack();
                let data: Cow<str> = match &input {
                    Value::Int(int) => int.to_string().into(),
                    Value::Float(float) => float.to_string().into(),
                    Value::Bool(boolean) => boolean.to_string().into(),
                    Value::String(string) => string.as_str().into(),
                    Value::Null => String::new().into(),
                    _ => {
                        return Err(ShellErrorKind::Basic(
                            "TypeError",
                            format!("Cannot cast a {} to a `string`", input.to_type()),
                        ))
                    }
                };
                save_file(path, data.as_bytes(), append)
            }
        }
    } else {
        Err(ShellErrorKind::Basic(
            "Serialization Error",
            "Cannot serialize without a format".into(),
        ))
    }
}
