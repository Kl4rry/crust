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
        .setting(clap::AppSettings::NoBinaryName)
        .try_get_matches_from(args.iter());

    let mut output = OutputStream::default();

    match matches {
        Ok(_) => output.stream.push(Value::String(String::from(
            std::env::current_dir().unwrap().to_str().unwrap(),
        ))),
        Err(err) => {
            eprintln!("{}", err);
            output.status = -1;
        }
    };
    Ok(output)
}
