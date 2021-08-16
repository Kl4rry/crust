use crate::error::SyntaxError;
use crate::lexer::token::{Token, TokenType};
use std::convert::TryFrom;

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
    Expand(String),     // Should be glob and variable expanded.
    Text(String),
    Char(char),
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
    Unary(UnOp, Box<Expr>),
}

#[derive(Debug)]
pub enum Literal {
    String(String),
    Expand(String),
    Number(f64),
    Bool(bool),
}

#[derive(Debug)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Expo,
    Mod,
    /// The == operator (equality)
    Eq,
    /// The < operator (less than)
    Lt,
    /// The <= operator (less than or equal to)
    Le,
    /// The != operator (not equal to)
    Ne,
    /// The >= operator (greater than or equal to)
    Ge,
    /// The > operator (greater than)
    Gt,
    And,
    Or,
}

#[derive(Debug)]
pub enum Command {
    Expand(String),
    String(String),
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
    Declaration(Variable, Option<Expr>),
    Assignment(Variable, Expr),
    If(Expr, Block),
    Fn(ArgumentList, Block),
    Loop(Block),
    While(Expr, Block),
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
            TokenType::Variable(name) => Ok(Variable {
                name: name.to_string(),
            }),
            _ => Err(SyntaxError::UnexpectedToken(token)),
        }
    }
}

impl TryFrom<Token> for BinOp {
    type Error = SyntaxError;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Mod => Ok(BinOp::Mod),
            TokenType::Eq => Ok(BinOp::Eq),
            TokenType::Lt => Ok(BinOp::Lt),
            TokenType::Le => Ok(BinOp::Le),
            TokenType::Ne => Ok(BinOp::Ne),
            TokenType::Ge => Ok(BinOp::Ge),
            TokenType::Gt => Ok(BinOp::Gt),
            TokenType::And => Ok(BinOp::And),
            TokenType::Or => Ok(BinOp::Or),
            _ => Err(SyntaxError::UnexpectedToken(token)),
        }
    }
}
