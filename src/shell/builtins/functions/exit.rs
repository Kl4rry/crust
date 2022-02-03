use crate::{
    parser::runtime_error::RunTimeError,
    shell::{
        stream::{OutputStream, ValueStream},
        Shell,
    },
};

pub fn exit(
    shell: &mut Shell,
    args: &[String],
    _: ValueStream,
) -> Result<OutputStream, RunTimeError> {
    let matches = clap::App::new("exit")
        .about("exit the shell")
        .arg(clap::Arg::with_name("STATUS").help("The exit status of the shell"))
        .settings(&[clap::AppSettings::NoBinaryName])
        .get_matches_from_safe(args.iter());

    let mut output = OutputStream::default();

    let matches = match matches {
        Ok(matches) => matches,
        Err(clap::Error { message, .. }) => {
            eprintln!("{}", message);
            output.status = -1;
            return Ok(output);
        }
    };

    if let Some(status) = matches.value_of("STATUS") {
        shell.exit_status = match status.to_string().parse() {
            Ok(number) => number,
            Err(_) => {
                eprintln!("exit: STATUS must be integer");
                output.status = -1;
                return Ok(output);
            }
        };
    }

    shell.running = false;
    Err(RunTimeError::Exit)
}
