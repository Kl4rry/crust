use crate::{parser::runtime_error::RunTimeError, shell::values::ValueKind, Shell};

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
    pub fn eval(&mut self, shell: &mut Shell) -> Result<Vec<ValueKind>, RunTimeError> {
        let mut values = Vec::new();
        for compound in &mut self.sequence {
            match compound {
                Compound::Expr(expr) => {
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

#[derive(Debug)]
pub enum Compound {
    Statement(Statement),
    Expr(Expr),
}

#[derive(Debug)]
pub struct Block {
    pub sequence: Vec<Compound>,
}

pub trait Precedence {
    fn precedence(&self) -> (u8, Direction);
}

#[derive(Debug, PartialEq)]
pub enum Direction {
    Left,
    Right,
}
