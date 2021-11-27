use std::{convert::TryFrom, fmt};

use crate::parser::{syntax_error::SyntaxErrorKind, Token, TokenType};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnOp {
    Neg,
    Not,
}

impl AsRef<str> for UnOp {
    fn as_ref(&self) -> &str {
        match self {
            Self::Neg => "-",
            Self::Not => "!",
        }
    }
}

impl fmt::Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
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
