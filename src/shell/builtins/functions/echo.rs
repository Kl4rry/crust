use thin_string::ToThinString;

use crate::{
    parser::runtime_error::RunTimeError,
    shell::{
        stream::{OutputStream, ValueStream},
        value::Value,
        Shell,
    },
};

pub fn echo(_: &mut Shell, args: &[String], _: ValueStream) -> Result<OutputStream, RunTimeError> {
    let mut output = OutputStream::default();
    for arg in args {
        output
            .stream
            .values
            .push_back(Value::String(arg.to_thin_string()));
    }
    Ok(output)
}
