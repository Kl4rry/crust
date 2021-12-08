use std::{convert::TryFrom, fmt};

use crate::parser::{
    ast::{Direction, Precedence},
    syntax_error::SyntaxErrorKind,
    Token, TokenType,
};

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
    /// The != operator (not equal to)
    Ne,
    /// The < operator (less than)
    Lt,
    /// The <= operator (less than or equal to)
    Le,
    /// The >= operator (greater than or equal to)
    Ge,
    /// The > operator (greater than)
    Gt,
    And,
    Or,
    Range,
}

impl AsRef<str> for BinOp {
    fn as_ref(&self) -> &str {
        match self {
            Self::Expo => "**",
            Self::Add => "+",
            Self::Sub => "-",
            Self::Mul => "*",
            Self::Div => "/",
            Self::Mod => "%",
            Self::Eq => "==",
            Self::Ne => "!=",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Ge => ">",
            Self::Gt => ">=",
            Self::And => "&&",
            Self::Or => "||",
            Self::Range => "..",
        }
    }
}

impl fmt::Display for BinOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
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
            Self::Sub => (6, Direction::Left),
            Self::Add => (5, Direction::Left),
            Self::Range => (4, Direction::Left),
            Self::Eq => (3, Direction::Left),
            Self::Lt => (3, Direction::Left),
            Self::Le => (3, Direction::Left),
            Self::Ne => (3, Direction::Left),
            Self::Ge => (3, Direction::Left),
            Self::Gt => (3, Direction::Left),
            Self::And => (2, Direction::Left),
            Self::Or => (2, Direction::Left),
        }
    }
}
