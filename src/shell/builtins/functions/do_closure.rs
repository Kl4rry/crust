use once_cell::sync::Lazy;

use crate::{
    argparse::{App, Arg, ParseResult},
    parser::{ast::context::Context, shell_error::ShellErrorKind},
    shell::{
        stream::ValueStream,
        value::{SpannedValue, Type},
    },
};

static APP: Lazy<App> = Lazy::new(|| {
    App::new("do")
        .about("Execute closure")
        .arg(
            Arg::new("CLOSURE", Type::CLOSURE)
                .required(true)
                .help("The closure to be executed"),
        )
        .arg(
            Arg::new("ARGUMENTS", Type::all())
                .multiple(true)
                .help("Arguments to be passed to closure"),
        )
});

pub fn do_closure(
    ctx: &mut Context,
    args: Vec<SpannedValue>,
    input: ValueStream,
) -> Result<(), ShellErrorKind> {
    let mut matches = match APP.parse(args) {
        Ok(ParseResult::Matches(m)) => m,
        Ok(ParseResult::Info(info)) => {
            ctx.output.push(info);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    let args = matches.take_values("ARGUMENTS").unwrap_or_default();
    let (closure, frame) = &*matches
        .take_value("CLOSURE")
        .unwrap()
        .value
        .unwrap_closure();

    closure.eval(ctx, frame.clone(), args.into_iter(), input)
}
