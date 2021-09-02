use crate::{parser::runtime_error::RunTimeError, shell::gc::Value, Shell};

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
    pub fn eval(&mut self, shell: &mut Shell) -> Result<(), RunTimeError> {
        for compound in &mut self.sequence {
            compound.eval(shell)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Compound {
    Statement(Statement),
    Expr(Expr),
}

impl Compound {
    pub fn eval(&mut self, shell: &mut Shell) -> Result<Option<Value>, RunTimeError> {
        match self {
            Compound::Expr(expr) => Ok(Some(expr.eval(shell)?)),
            Compound::Statement(statement) => {
                statement.eval(shell)?;
                Ok(None)
            }
        }
    }
}

#[derive(Debug)]
pub struct Block {
    pub sequence: Vec<Compound>,
}

pub trait Precedence {
    fn precedence(&self) -> u8;
}
