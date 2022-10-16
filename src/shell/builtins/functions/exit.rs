use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Arg, ParseResult},
    parser::shell_error::ShellErrorKind,
    shell::{
        frame::Frame,
        stream::{OutputStream, ValueStream},
        value::{Type, Value},
        Shell,
    },
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("exit")
        .about("Exit the shell")
        .arg(Arg::new("STATUS", Type::INT).help("The exit status of the shell"))
});

pub fn exit(
    shell: &mut Shell,
    _: &mut Frame,
    args: Vec<Value>,
    _: ValueStream,
    output: &mut OutputStream,
) -> Result<(), ShellErrorKind> {
    let matches = match APP.parse(args.into_iter()) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let status = matches.value("STATUS").unwrap_or(&Value::Int(0));
    shell.exit_status = status.unwrap_int();

    shell.running = false;
    Err(ShellErrorKind::Exit)
}
