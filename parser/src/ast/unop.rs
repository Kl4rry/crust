use std::convert::TryFrom;

use crate::{ast::Precedence, SyntaxError, Token, TokenType};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnOp {
    Neg,
    Not,
}

impl TryFrom<Token> for UnOp {
    type Error = SyntaxError;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Not => Ok(UnOp::Not),
            TokenType::Sub => Ok(UnOp::Neg),
            _ => Err(SyntaxError::UnexpectedToken(token)),
        }
    }
}

impl Precedence for UnOp {
    fn precedence(&self) -> u8 {
        match *self {
            Self::Neg => 10,
            Self::Not => 10,
        }
    }
}
