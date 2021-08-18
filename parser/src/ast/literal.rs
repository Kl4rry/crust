use std::convert::TryFrom;

use crate::{SyntaxError, Token, TokenType};

#[derive(Debug)]
pub enum Literal {
    String(String),
    Expand(String),
    Float(f64),
    Int(u128),
    Bool(bool),
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
