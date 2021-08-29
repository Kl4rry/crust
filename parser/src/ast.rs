use std::convert::TryFrom;

use crate::{
    error::SyntaxErrorKind,
    lexer::token::{Token, TokenType},
    P,
};

pub mod literal;
use literal::Literal;

pub mod expr;
use expr::Expr;

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
    Expand(Expand),     // Should be variable expanded.
    Glob(String),
    String(String),
    Expr(P<Expr>),
}

#[derive(Debug)]
pub struct Expand {
    pub content: Vec<ExpandKind>,
}

#[derive(Debug)]
pub enum ExpandKind {
    String(String),
    Expr(P<Expr>),
    Variable(Variable),
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
}

#[derive(Debug)]
pub struct Argument {
    pub parts: Vec<Identifier>,
}

#[derive(Debug)]
pub enum Statement {
    Export(Variable, Option<Expr>),
    Declaration(Variable, Option<Expr>),
    Assignment(Variable, Expr),
    Alias(Argument, Expr),
    If(Expr, Block, Option<P<Statement>>),
    Fn(String, Vec<Variable>, Block),
    Return(Option<Expr>),
    Loop(Block),
    While(Expr, Block),
    For(Variable, Expr, Block),
    Break,
    Continue,
    Block(Block),
}

#[derive(Debug)]
pub struct Block {
    pub sequence: Vec<Compound>,
}

impl TryFrom<Token> for Variable {
    type Error = SyntaxErrorKind;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Variable(name) => Ok(Variable { name }),
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }
}

pub trait Precedence {
    fn precedence(&self) -> u8;
}
