use std::convert::TryFrom;

use crate::parser::{
    ast::{Direction, Precedence},
    syntax_error::SyntaxErrorKind,
    Token, TokenType,
};

#[derive(Debug, PartialEq, Eq)]
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
    Range,
}

impl BinOp {
    pub fn is_comparison(&self) -> bool {
        matches!(
            *self,
            Self::Eq | Self::Lt | Self::Le | Self::Ne | Self::Ge | Self::Gt
        )
    }
}

impl TryFrom<Token> for BinOp {
    type Error = SyntaxErrorKind;
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
            TokenType::Range => Ok(BinOp::Range),
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }
}

impl Precedence for BinOp {
    fn precedence(&self) -> (u8, Direction) {
        match self {
            Self::Expo => (9, Direction::Right),
            Self::Mul => (8, Direction::Left),
            Self::Div => (7, Direction::Left),
            Self::Mod => (7, Direction::Left),
            Self::Add => (6, Direction::Left),
            Self::Sub => (6, Direction::Left),
            Self::Eq => (5, Direction::Left),
            Self::Lt => (5, Direction::Left),
            Self::Le => (5, Direction::Left),
            Self::Ne => (5, Direction::Left),
            Self::Ge => (5, Direction::Left),
            Self::Gt => (5, Direction::Left),
            Self::And => (4, Direction::Left),
            Self::Or => (4, Direction::Left),
            Self::Range => (3, Direction::Left),
        }
    }
}
