use crate::{
    parser::runtime_error::RunTimeError,
    shell::{values::ValueKind, Frame},
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
    pub fn eval(&self, shell: &mut Shell) -> Result<Vec<ValueKind>, RunTimeError> {
        let mut values = Vec::new();
        for compound in &self.sequence {
            match compound {
                Compound::Expr(expr) => {
                    // this is a wacky hack to avoid echoing status codes while still echoing other values
                    if matches!(expr, Expr::Call(..)) {
                        expr.eval(shell, false)?;
                    } else {
                        values.push(expr.eval(shell, false)?);
                    }
                }
                Compound::Statement(statement) => {
                    statement.eval(shell)?;
                }
            }
        }
        Ok(values)
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
    pub fn eval(&self, shell: &mut Shell) -> Result<(), RunTimeError> {
        shell.stack.push(Frame::new());
        for compound in &self.sequence {
            match compound {
                Compound::Expr(expr) => {
                    expr.eval(shell, false)?;
                }
                Compound::Statement(statement) => {
                    statement.eval(shell)?;
                }
            }
        }
        shell.stack.pop();
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
