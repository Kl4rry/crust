use std::convert::{TryFrom, TryInto};

use crate::{
    ast::{Expand, Variable},
    error::SyntaxErrorKind,
    lexer::token::{Token, TokenType},
};

#[derive(Debug)]
pub enum Command {
    Expand(Expand),
    String(String),
    Variable(Variable),
}

impl TryFrom<Token> for Command {
    type Error = SyntaxErrorKind;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::String(text) => Ok(Command::String(text)),
            TokenType::Symbol(text) => Ok(Command::String(text)),
            TokenType::Int(_, text) => Ok(Command::String(text)),
            TokenType::Float(_, text) => Ok(Command::String(text)),
            TokenType::Variable(_) => Ok(Command::Variable(token.try_into()?)),
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }
}
