use crate::parser::{
    ast::{Argument, Literal, Variable},
    P,
};

use crate::shell::gc::Value;

pub mod binop;
use binop::BinOp;

pub mod unop;
use unop::UnOp;

pub mod command;
use command::Command;

#[derive(Debug)]
pub enum Direction {
    Left,
    Right,
}

#[derive(Debug)]
pub enum Expr {
    Call(Command, Vec<Argument>),
    Pipe(P<Expr>, P<Expr>),
    Redirect(Direction, P<Expr>, Argument),
    Variable(Variable),
    Binary(BinOp, P<Expr>, P<Expr>),
    Paren(P<Expr>),
    Unary(UnOp, P<Expr>),
    Literal(Literal),
}

impl Expr {
    #[inline]
    pub fn wrap(self, unop: Option<UnOp>) -> Self {
        match unop {
            Some(unop) => Expr::Unary(unop, P::new(self)),
            None => self,
        }
    }

    pub fn evaluate(&self) -> Value {
        todo!();
    }
}
