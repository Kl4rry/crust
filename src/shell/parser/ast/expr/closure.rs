use std::{collections::HashMap, sync::Arc};

use crate::{
    parser::{
        ast::{context::Context, variable::Variable, Block},
        lexer::token::span::Span,
        shell_error::ShellErrorKind,
        source::Source,
    },
    shell::{frame::Frame, stream::ValueStream, value::SpannedValue},
};

#[derive(Debug)]
pub struct Closure {
    pub span: Span,
    pub arg_span: Span,
    pub parameters: Vec<Variable>,
    pub block: Block,
    pub src: Arc<Source>,
}

impl Closure {
    pub fn eval(
        &self,
        ctx: &mut Context,
        frame: Frame,
        arguments: impl ExactSizeIterator<Item = SpannedValue>,
        input: ValueStream,
    ) -> Result<(), ShellErrorKind> {
        let Closure {
            parameters,
            block,
            src,
            arg_span,
            ..
        } = self;

        if parameters.len() != arguments.len() {
            return Err(ShellErrorKind::IncorrectArgumentCount {
                name: None,
                expected: parameters.len(),
                recived: arguments.len(),
                arg_span: *arg_span,
                src: src.clone(),
            });
        }

        let mut input_vars = HashMap::new();
        for (var, arg) in parameters.iter().zip(arguments) {
            input_vars.insert(var.name.clone(), (false, arg.clone().value));
        }

        let ctx = &mut Context {
            shell: ctx.shell,
            frame: frame.clone(),
            output: ctx.output,
            src: ctx.src.clone(),
        };
        block.eval(ctx, Some(input_vars), Some(input))
    }
}
