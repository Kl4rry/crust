use std::convert::TryFrom;

use crate::{
    error::SyntaxError,
    lexer::token::{Token, TokenType},
    Small,
};

pub mod binop;
use binop::BinOp;

pub mod unop;
use unop::UnOp;

pub mod literal;
use literal::Literal;

#[derive(Debug)]
pub struct Ast {
    pub sequence: Vec<Compound>,
}

#[derive(Debug)]
pub enum Compound {
    Statement(Statement),
    Expr(Expr),
}

#[derive(Debug)]
pub enum Identifier {
    Variable(Variable), // Should be expaned to variable value. Must be done before glob.
    Expand(Small),      // Should be glob and variable expanded.
    Glob(Small),
    Text(Small),
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
}

#[derive(Debug)]
pub enum Expr {
    Call(Command, Vec<Argument>),
    Variable(Variable),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Paren(Box<Expr>),
    Unary(UnOp, Box<Expr>),
    Literal(Literal),
}

#[derive(Debug)]
pub enum Command {
    Expand(String),
    String(String),
    Variable(Variable),
}

#[derive(Debug)]
pub struct Pipe {
    pub source: Expr,
    pub destination: Expr,
}

#[derive(Debug)]
pub struct Argument {
    pub parts: Vec<Identifier>,
}

#[derive(Debug)]
pub struct Redirect {
    pub call: Expr,
    pub file: Identifier,
}

#[derive(Debug)]
pub enum Statement {
    Export(Variable, Option<Expr>),
    Declaration(Variable, Option<Expr>),
    Assignment(Variable, Expr),
    If(Expr, Block),
    Fn(ArgumentList, Block),
    Loop(Block),
    While(Expr, Block),
    Break,
}

#[derive(Debug)]
pub struct ArgumentList {
    args: Vec<String>,
}

#[derive(Debug)]
pub enum Block {}

impl TryFrom<Token> for Variable {
    type Error = SyntaxError;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Variable(name) => Ok(Variable { name }),
            _ => Err(SyntaxError::UnexpectedToken(token)),
        }
    }
}

pub trait Precedence {
    fn precedence(&self) -> u8 {
        0
    }
}
