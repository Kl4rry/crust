use std::{convert::TryFrom, fmt};

use crate::parser::{
    ast::{Direction, Precedence},
    lexer::token::span::Span,
    syntax_error::SyntaxErrorKind,
    Token, TokenType,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinOpKind {
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
    /// The =~ operator
    Match,
    /// The !~ operator
    NotMatch,
}

impl BinOpKind {
    pub fn spanned(self, span: Span) -> BinOp {
        BinOp { kind: self, span }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BinOp {
    pub kind: BinOpKind,
    pub span: Span,
}

impl AsRef<str> for BinOp {
    fn as_ref(&self) -> &str {
        use BinOpKind::*;
        match self.kind {
            Expo => "**",
            Add => "+",
            Sub => "-",
            Mul => "*",
            Div => "/",
            Mod => "%",
            Eq => "==",
            Ne => "!=",
            Lt => "<",
            Le => "<=",
            Ge => ">",
            Gt => ">=",
            And => "&&",
            Or => "||",
            Range => "..",
            Match => "=~",
            NotMatch => "!~",
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
        use BinOpKind::*;
        matches!(self.kind, Eq | Lt | Le | Ne | Ge | Gt)
    }
}

impl TryFrom<Token> for BinOp {
    type Error = SyntaxErrorKind;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Add => Ok(BinOpKind::Add.spanned(token.span)),
            TokenType::Sub => Ok(BinOpKind::Sub.spanned(token.span)),
            TokenType::Mul => Ok(BinOpKind::Mul.spanned(token.span)),
            TokenType::Div => Ok(BinOpKind::Div.spanned(token.span)),
            TokenType::Expo => Ok(BinOpKind::Expo.spanned(token.span)),
            TokenType::Mod => Ok(BinOpKind::Mod.spanned(token.span)),
            TokenType::Eq => Ok(BinOpKind::Eq.spanned(token.span)),
            TokenType::Lt => Ok(BinOpKind::Lt.spanned(token.span)),
            TokenType::Le => Ok(BinOpKind::Le.spanned(token.span)),
            TokenType::Ne => Ok(BinOpKind::Ne.spanned(token.span)),
            TokenType::Ge => Ok(BinOpKind::Ge.spanned(token.span)),
            TokenType::Gt => Ok(BinOpKind::Gt.spanned(token.span)),
            TokenType::And => Ok(BinOpKind::And.spanned(token.span)),
            TokenType::Or => Ok(BinOpKind::Or.spanned(token.span)),
            TokenType::Range => Ok(BinOpKind::Range.spanned(token.span)),
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }
}

impl Precedence for BinOp {
    fn precedence(&self) -> (u8, Direction) {
        use BinOpKind::*;
        match self.kind {
            Expo => (10, Direction::Right),

            Match => (9, Direction::Left),
            NotMatch => (9, Direction::Left),

            Mul => (8, Direction::Left),
            Div => (8, Direction::Left),
            Mod => (8, Direction::Left),
            Sub => (7, Direction::Left),
            Add => (7, Direction::Left),
            Range => (6, Direction::Left),

            Lt => (5, Direction::Left),
            Le => (5, Direction::Left),
            Ge => (5, Direction::Left),
            Gt => (5, Direction::Left),

            Eq => (4, Direction::Left),
            Ne => (4, Direction::Left),

            And => (3, Direction::Left),
            Or => (2, Direction::Left),
        }
    }
}
