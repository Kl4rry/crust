use std::{convert::TryFrom, fmt};

use crate::parser::{lexer::token::span::Span, syntax_error::SyntaxErrorKind, Token, TokenType};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AssignOpKind {
    Expo,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl AssignOpKind {
    pub fn spanned(self, span: Span) -> AssignOp {
        AssignOp { kind: self, span }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AssignOp {
    pub kind: AssignOpKind,
    pub span: Span,
}

impl AsRef<str> for AssignOp {
    fn as_ref(&self) -> &str {
        use AssignOpKind::*;
        match self.kind {
            Expo => "**=",
            Add => "+=",
            Sub => "-=",
            Mul => "*=",
            Div => "/=",
            Mod => "%=",
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
            TokenType::Add => Ok(AssignOpKind::Add.spanned(token.span)),
            TokenType::Sub => Ok(AssignOpKind::Sub.spanned(token.span)),
            TokenType::Mul => Ok(AssignOpKind::Mul.spanned(token.span)),
            TokenType::Div => Ok(AssignOpKind::Div.spanned(token.span)),
            TokenType::Expo => Ok(AssignOpKind::Expo.spanned(token.span)),
            TokenType::Mod => Ok(AssignOpKind::Mod.spanned(token.span)),
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }
}
