use std::{collections::HashMap, sync::atomic::Ordering};

use crate::{
    parser::shell_error::ShellErrorKind,
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

use super::shell_error::ShellError;

#[derive(Debug)]
pub struct Ast {
    sequence: Vec<Compound>,
    src: String,
    name: String,
}

impl Ast {
    pub fn new(sequence: Vec<Compound>, src: String, name: String) -> Self {
        Self {
            sequence,
            src,
            name,
        }
    }

    pub fn eval(&self, shell: &mut Shell) -> Result<OutputStream, ShellError> {
        let x = self.eval_errorkind(shell);
        x.map_err(|err| ShellError::new(err, self.src.clone(), self.name.clone()))
    }

    pub fn eval_errorkind(&self, shell: &mut Shell) -> Result<OutputStream, ShellErrorKind> {
        let mut output = OutputStream::default();
        for compound in &self.sequence {
            if shell.interrupt.load(Ordering::SeqCst) {
                return Err(ShellErrorKind::Interrupt);
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
    ) -> Result<OutputStream, ShellErrorKind> {
        if shell.stack.len() == shell.recursion_limit {
            return Err(ShellErrorKind::MaxRecursion(shell.recursion_limit));
        }
        shell.stack.push(Frame::new(
            variables.unwrap_or_default(),
            HashMap::new(),
            input.unwrap_or_default(),
        ));
        let mut output = OutputStream::default();
        for compound in &self.sequence {
            if shell.interrupt.load(Ordering::SeqCst) {
                return Err(ShellErrorKind::Interrupt);
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
