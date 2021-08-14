use super::error::SyntaxError;
use super::lexer::token::{Token, TokenType};
use std::convert::TryFrom;

#[derive(Debug)]
pub struct Ast {
    pub sequence: Vec<Compound>,
}

#[derive(Debug)]
pub enum Compound {
    Statement(Statement),
    Expression(Expression),
}

#[derive(Debug)]
pub enum Identifier {
    Variable(Variable), // Should be expaned to variable value. Must be done before glob.
    Expand(String),     // Should be glob and variable expanded.
    Text(String),
    Char(char),
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
}

#[derive(Debug)]
pub enum Expression {
    Call(Command, Vec<Argument>),
    Variable(Variable),
    Redirect, // ?
    Pipe,
    Add,
    Sub,
}

#[derive(Debug)]
pub enum Command {
    Expand(String),
    Text(String),
}

#[derive(Debug)]
pub struct Pipe {
    pub source: Expression,
    pub destination: Expression,
}

#[derive(Debug)]
pub struct Argument {
    pub parts: Vec<Identifier>,
}

#[derive(Debug)]
pub struct Redirect {
    pub call: Expression,
    pub file: Identifier,
}

#[derive(Debug)]
pub enum Statement {
    Assignment(Variable, Expression),
    _If,
    _Fn,
    _Loop,
}

impl TryFrom<Token> for Variable {
    type Error = SyntaxError;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Variable(name) => Ok(Variable { name }),
            _ => Err(SyntaxError::UnexpectedToken(token)),
        }
    }
}
