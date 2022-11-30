use std::{convert::TryFrom, fmt};

use crate::parser::{lexer::token::span::Span, syntax_error::SyntaxErrorKind, Token, TokenType};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnOpKind {
    Neg,
    Not,
}

impl UnOpKind {
    pub fn spanned(self, span: Span) -> UnOp {
        UnOp { kind: self, span }
    }
}

impl AsRef<str> for UnOpKind {
    fn as_ref(&self) -> &str {
        match self {
            Self::Neg => "-",
            Self::Not => "!",
        }
    }
}

impl fmt::Display for UnOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.kind.as_ref())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UnOp {
    pub kind: UnOpKind,
    pub span: Span,
}

impl TryFrom<Token> for UnOp {
    type Error = SyntaxErrorKind;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Not => Ok(UnOpKind::Not.spanned(token.span)),
            TokenType::Sub => Ok(UnOpKind::Neg.spanned(token.span)),
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }
}
