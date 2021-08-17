use crate::error::SyntaxError;
use crate::lexer::token::{Token, TokenType};
use crate::Small;
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
    Expand(Small),     // Should be glob and variable expanded.
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
    Unary(UnOp, Box<Expr>),
    Literal(Literal),
}

#[derive(Debug)]
pub enum Literal {
    String(String),
    Expand(String),
    Float(f64),
    Int(u128),
    Bool(bool),
}

#[repr(u8)]
#[derive(Debug)]
pub enum UnOp {
    Neg = 0,
    Not = 1,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum BinOp {
    Expo = 2,
    Add,
    Sub,
    Mul,
    Div,
    
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

impl TryFrom<Token> for Literal {
    type Error = SyntaxError;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::String(text) => Ok(Literal::String(text)),
            TokenType::ExpandString(text) => Ok(Literal::Expand(text)),
            TokenType::Float(number, _) => Ok(Literal::Float(number)),
            TokenType::Int(number, _) => Ok(Literal::Int(number)),
            TokenType::Symbol(text) => Ok(Literal::Bool(text.parse().unwrap())),
            _ => Err(SyntaxError::UnexpectedToken(token)),
        }
    }
}

impl TryFrom<Token> for BinOp {
    type Error = SyntaxError;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Add => Ok(BinOp::Add),
            TokenType::Sub => Ok(BinOp::Sub),
            TokenType::Mul => Ok(BinOp::Mul),
            TokenType::Div => Ok(BinOp::Div),
            TokenType::Expo => Ok(BinOp::Expo),
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

impl TryFrom<Token> for UnOp {
    type Error = SyntaxError;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Add => Ok(UnOp::Not),
            TokenType::Sub => Ok(UnOp::Neg),
            _ => Err(SyntaxError::UnexpectedToken(token)),
        }
    }
}
