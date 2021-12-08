use std::{convert::TryFrom, fmt};

use crate::parser::{syntax_error::SyntaxErrorKind, Token, TokenType};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AssignOp {
    Expo,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl AsRef<str> for AssignOp {
    fn as_ref(&self) -> &str {
        match self {
            Self::Expo => "**=",
            Self::Add => "+=",
            Self::Sub => "-=",
            Self::Mul => "*=",
            Self::Div => "/=",
            Self::Mod => "%=",
        }
    }
}

impl fmt::Display for AssignOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl TryFrom<Token> for AssignOp {
    type Error = SyntaxErrorKind;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Add => Ok(AssignOp::Add),
            TokenType::Sub => Ok(AssignOp::Sub),
            TokenType::Mul => Ok(AssignOp::Mul),
            TokenType::Div => Ok(AssignOp::Div),
            TokenType::Expo => Ok(AssignOp::Expo),
            TokenType::Mod => Ok(AssignOp::Mod),
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }
}
