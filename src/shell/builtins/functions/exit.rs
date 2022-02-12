use crate::{
    parser::shell_error::ShellError,
    shell::{
        stream::{OutputStream, ValueStream},
        Shell,
    },
};

pub fn exit(
    shell: &mut Shell,
    args: &[String],
    _: ValueStream,
) -> Result<OutputStream, ShellError> {
    let matches = clap::App::new("exit")
        .about("exit the shell")
        .arg(clap::Arg::new("STATUS").help("The exit status of the shell"))
        .setting(clap::AppSettings::NoBinaryName)
        .try_get_matches_from(args.iter());

    let mut output = OutputStream::default();

    let matches = match matches {
        Ok(matches) => matches,
        Err(err) => {
            eprintln!("{}", err);
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
    Err(ShellError::Exit)
}
