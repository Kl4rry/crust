use thin_string::ThinString;

use crate::{
    parser::runtime_error::RunTimeError,
    shell::{
        stream::{OutputStream, ValueStream},
        Shell, Value,
    },
};

pub fn pwd(_: &mut Shell, args: &[String], _: ValueStream) -> Result<OutputStream, RunTimeError> {
    let matches = clap::App::new("pwd")
        .about("print working directory")
        .settings(&[clap::AppSettings::NoBinaryName])
        .get_matches_from_safe(args.iter());

    let mut output = OutputStream::default();

    match matches {
        Ok(_) => output.stream.push(Value::String(ThinString::from(
            std::env::current_dir().unwrap().to_str().unwrap(),
        ))),
        Err(clap::Error { message, .. }) => {
            eprintln!("{}", message);
            output.status = -1;
        }
    };
    Ok(output)
}
