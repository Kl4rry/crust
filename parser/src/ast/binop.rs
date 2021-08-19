use std::convert::TryFrom;

use crate::{ast::Precedence, SyntaxError, Token, TokenType};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinOp {
    Expo,
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

impl Precedence for BinOp {
    fn precedence(&self) -> u8 {
        match self {
            Self::Expo => 9,
            Self::Mul => 8,
            Self::Div => 8,
            Self::Mod => 8,
            Self::Add => 7,
            Self::Sub => 7,
            Self::Eq => 6,
            Self::Lt => 6,
            Self::Le => 6,
            Self::Ne => 6,
            Self::Ge => 6,
            Self::Gt => 6,
            Self::And => 5,
            Self::Or => 5,
        }
    }
}
