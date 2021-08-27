use std::convert::TryFrom;

use crate::{ast::Expr, error::SyntaxErrorKind, Token, TokenType};

#[derive(Debug)]
pub enum Literal {
    String(String),
    Expand(String),
    List(Vec<Expr>),
    Float(f64),
    Int(u128),
    Bool(bool),
}

impl TryFrom<Token> for Literal {
    type Error = SyntaxErrorKind;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::String(text) => Ok(Literal::String(text)),
            TokenType::ExpandString(text) => Ok(Literal::Expand(text)),
            TokenType::Float(number, _) => Ok(Literal::Float(number)),
            TokenType::Int(number, _) => Ok(Literal::Int(number)),
            TokenType::True => Ok(Literal::Bool(true)),
            TokenType::False => Ok(Literal::Bool(false)),
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }
}
