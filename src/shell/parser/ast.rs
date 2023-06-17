use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Arc},
};

use crate::{
    parser::shell_error::ShellErrorKind,
    shell::{
        stream::{OutputStream, ValueStream},
        value::Value,
    },
    Shell,
};

pub mod literal;
use literal::Literal;

pub mod expr;
use expr::Expr;

pub mod statement;

pub mod variable;
use variable::Variable;

pub mod context;

use self::{context::Context, statement::Statement};
use super::{lexer::token::span::Span, shell_error::ShellError, source::Source};

#[derive(Debug)]
pub struct Ast {
    sequence: Vec<Compound>,
    src: Arc<Source>,
}

impl Ast {
    pub fn new(sequence: Vec<Compound>, src: Arc<Source>) -> Self {
        Self { sequence, src }
    }

    pub fn eval(&self, shell: &mut Shell, output: &mut OutputStream) -> Result<(), ShellError> {
        let res = self.eval_errorkind(shell, output);
        res.map_err(|err| ShellError::new(err, (*self.src).clone().into()))
    }

    pub fn eval_errorkind(
        &self,
        shell: &mut Shell,
        output: &mut OutputStream,
    ) -> Result<(), ShellErrorKind> {
        for compound in &self.sequence {
            if shell.interrupt.load(Ordering::SeqCst) {
                return Err(ShellErrorKind::Interrupt);
            }
            let frame = shell.stack.clone();
            let mut ctx = Context {
                shell,
                frame,
                output,
                src: self.src.clone(),
            };

            match compound {
                Compound::Expr(expr) => {
                    let value = expr.eval(&mut ctx)?;
                    output.push(value.into());
                }
                Compound::Statement(statement) => statement.eval(&mut ctx)?,
            };
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum Compound {
    Statement(Statement),
    Expr(Expr),
}

impl Compound {
    pub fn span(&self) -> Span {
        match self {
            Self::Statement(statement) => statement.span,
            Self::Expr(expr) => expr.span,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Block {
    pub sequence: Vec<Compound>,
    pub span: Span,
}

impl Block {
    pub fn eval(
        &self,
        ctx: &mut Context,
        variables: Option<HashMap<String, (bool, Value)>>,
        input: Option<ValueStream>,
    ) -> Result<(), ShellErrorKind> {
        if ctx.frame.index() == ctx.shell.recursion_limit {
            return Err(ShellErrorKind::MaxRecursion(ctx.shell.recursion_limit));
        }
        let pre_block_dir = std::env::current_dir().map_err(|err| ShellErrorKind::Io(None, err))?;
        let frame = ctx.frame.clone().push(
            variables.unwrap_or_default(),
            HashMap::new(),
            input.unwrap_or_default(),
        );
        let ctx = &mut Context {
            shell: ctx.shell,
            frame,
            output: ctx.output,
            src: ctx.src.clone(),
        };
        for compound in &self.sequence {
            if ctx.shell.interrupt.load(Ordering::SeqCst) {
                return Err(ShellErrorKind::Interrupt);
            }
            match compound {
                Compound::Expr(expr) => {
                    let value = expr.eval(ctx)?;
                    ctx.output.push(value.into());
                }
                Compound::Statement(statement) => statement.eval(ctx)?,
            };
        }
        std::env::set_current_dir(pre_block_dir).map_err(|err| ShellErrorKind::Io(None, err))
    }
}

pub trait Precedence {
    fn precedence(&self) -> (u8, Direction);
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
}
