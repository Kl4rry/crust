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

    pub fn eval(&self, shell: &mut Shell, output: &mut OutputStream) -> Result<(), ShellError> {
        let x = self.eval_errorkind(shell, output);
        x.map_err(|err| ShellError::new(err, self.src.clone(), self.name.clone()))
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
            match compound {
                Compound::Expr(expr) => {
                    let value = expr.eval(shell, output)?;
                    output.push(value);
                }
                Compound::Statement(statement) => statement.eval(shell, output)?,
            };
        }
        Ok(())
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
        output: &mut OutputStream,
    ) -> Result<(), ShellErrorKind> {
        if shell.stack.len() == shell.recursion_limit {
            return Err(ShellErrorKind::MaxRecursion(shell.recursion_limit));
        }
        shell.stack.push(Frame::new(
            variables.unwrap_or_default(),
            HashMap::new(),
            input.unwrap_or_default(),
        ));
        for compound in &self.sequence {
            if shell.interrupt.load(Ordering::SeqCst) {
                return Err(ShellErrorKind::Interrupt);
            }
            match compound {
                Compound::Expr(expr) => {
                    let value = expr.eval(shell, output)?;
                    output.push(value);
                }
                Compound::Statement(statement) => statement.eval(shell, output)?,
            };
        }
        shell.stack.pop().unwrap();
        Ok(())
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
