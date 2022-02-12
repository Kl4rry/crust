use std::{collections::HashMap, sync::atomic::Ordering};

use crate::{
    parser::shell_error::ShellError,
    shell::{
        stream::{OutputStream, ValueStream},
        value::Value,
        Frame,
    },
    Shell,
};

pub mod literal;
use literal::Literal;

pub mod expr;
use expr::Expr;

pub mod statement;
use statement::Statement;

pub mod variable;
use variable::Variable;

#[derive(Debug)]
pub struct Ast {
    pub sequence: Vec<Compound>,
}

impl Ast {
    pub fn eval(&self, shell: &mut Shell) -> Result<OutputStream, ShellError> {
        let mut output = OutputStream::default();
        for compound in &self.sequence {
            if shell.interrupt.load(Ordering::SeqCst) {
                return Err(ShellError::Interrupt);
            }
            let value = match compound {
                Compound::Expr(expr) => expr.eval(shell, false)?,
                Compound::Statement(statement) => statement.eval(shell)?,
            };
            match value {
                Value::Null => (),
                Value::OutputStream(stream) => output.extend(stream.into_iter()),
                value => output.push(value),
            }
        }
        Ok(output)
    }
}

#[derive(Debug, Clone)]
pub enum Compound {
    Statement(Statement),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub struct Block {
    pub sequence: Vec<Compound>,
}

impl Block {
    pub fn eval(
        &self,
        shell: &mut Shell,
        variables: Option<HashMap<String, Value>>,
        input: Option<ValueStream>,
    ) -> Result<OutputStream, ShellError> {
        if shell.stack.len() == shell.recursion_limit {
            return Err(ShellError::MaxRecursion(shell.recursion_limit));
        }
        shell.stack.push(Frame::new(
            variables.unwrap_or_default(),
            HashMap::new(),
            input.unwrap_or_default(),
        ));
        let mut output = OutputStream::default();
        for compound in &self.sequence {
            if shell.interrupt.load(Ordering::SeqCst) {
                return Err(ShellError::Interrupt);
            }
            let value = match compound {
                Compound::Expr(expr) => expr.eval(shell, false)?,
                Compound::Statement(statement) => statement.eval(shell)?,
            };
            match value {
                Value::Null => (),
                Value::OutputStream(stream) => output.extend(stream.into_iter()),
                value => output.push(value),
            }
        }
        shell.stack.pop().unwrap();
        Ok(output)
    }
}

pub trait Precedence {
    fn precedence(&self) -> (u8, Direction);
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
}
