use std::convert::TryFrom;

use crate::parser::{ast::Precedence, error::SyntaxErrorKind, Token, TokenType};

#[derive(Debug, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

impl TryFrom<Token> for UnOp {
    type Error = SyntaxErrorKind;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Not => Ok(UnOp::Not),
            TokenType::Sub => Ok(UnOp::Neg),
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
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
