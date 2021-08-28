pub mod span;

use std::{convert::TryInto, mem};

use span::Span;

use crate::{ast::Identifier, error::SyntaxErrorKind, Result};

#[derive(PartialEq, Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub span: Span,
}

#[derive(PartialEq, Debug, Clone)]
pub enum TokenType {
    Symbol(String),
    Variable(String),
    String(String),
    Float(f64, String),
    Int(u128, String),
    Quote,
    NewLine,
    Space,
    Exec,
    Assignment,
    Pipe,
    RightBrace,
    LeftBrace,
    RightParen,
    LeftParen,
    RightBracket,
    LeftBracket,
    Comma,
    /// $
    Dollar,
    //Colon,
    SemiColon,
    // Binary operators
    /// The x..y operator (range)
    Range,
    Add,
    Sub,
    Mul,
    Div,
    Expo,
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
    // Unary operators
    Not,

    // keywords
    If,
    Else,
    While,
    Loop,
    For,
    In,
    Break,
    Return,
    Continue,
    Fn,
    True,
    False,
}

impl TokenType {
    pub fn is_space(&self) -> bool {
        matches!(*self, Self::Space)
    }

    pub fn is_unop(&self) -> bool {
        matches!(*self, Self::Not | Self::Sub)
    }

    pub fn is_binop(&self) -> bool {
        matches!(
            *self,
            Self::Space
                | Self::Add
                | Self::Sub
                | Self::Mul
                | Self::Div
                | Self::Expo
                | Self::Mod
                | Self::Eq
                | Self::Lt
                | Self::Le
                | Self::Ne
                | Self::Ge
                | Self::Gt
                | Self::And
                | Self::Or
                | Self::Range
        )
    }

    pub fn is_valid_id(&self) -> bool {
        matches!(
            *self,
            TokenType::Dollar |
            TokenType::Quote |
            TokenType::Assignment |
            //TokenType::Colon |
            TokenType::RightBracket |
            TokenType::LeftBracket |
            TokenType::Range |
            TokenType::Add |
            TokenType::Sub |
            TokenType::Mul |
            TokenType::Div |
            TokenType::Expo |
            TokenType::Mod |
            TokenType::Eq  |
            TokenType::Le  |
            TokenType::Ne |
            TokenType::Ge  |
            TokenType::Not |
            TokenType::Symbol(_) |
            TokenType::Variable(_) |
            TokenType::String(_) |
            TokenType::Float(_, _) |
            TokenType::Int(_, _) |
            TokenType::True |
            TokenType::False
        )
    }
}

impl Token {
    pub fn expect(self, token_type: TokenType) -> Result<()> {
        if mem::discriminant(&self.token_type) == mem::discriminant(&token_type) {
            Ok(())
        } else {
            Err(SyntaxErrorKind::UnexpectedToken(self))
        }
    }

    pub fn is_space(&self) -> bool {
        self.token_type.is_space()
    }

    pub fn is_binop(&self) -> bool {
        self.token_type.is_binop()
    }

    pub fn is_unop(&self) -> bool {
        self.token_type.is_unop()
    }

    pub fn is_valid_id(&self) -> bool {
        self.token_type.is_valid_id()
    }

    pub fn try_into_id(self) -> Result<Identifier> {
        match self.token_type {
            TokenType::String(text) => Ok(Identifier::String(text)),
            TokenType::Symbol(text) => Ok(Identifier::Glob(text)),
            TokenType::Variable(_) => Ok(Identifier::Variable(self.try_into()?)),
            TokenType::Int(_, text) => Ok(Identifier::Glob(text)),
            TokenType::Float(_, text) => Ok(Identifier::Glob(text)),
            _ => return Ok(Identifier::Glob(self.try_into_glob_str()?.into())),
        }
    }

    pub fn try_into_glob_str(self) -> Result<&'static str> {
        match self.token_type {
            TokenType::Assignment => Ok("="),
            TokenType::RightBracket => Ok("]"),
            TokenType::LeftBracket => Ok("["),
            TokenType::Add => Ok("+"),
            TokenType::Sub => Ok("-"),
            TokenType::Div => Ok("/"),
            TokenType::Expo => Ok("^"),
            TokenType::Mod => Ok("%"),
            TokenType::Eq => Ok("=="),
            TokenType::Le => Ok("<="),
            TokenType::Ne => Ok("-"),
            TokenType::Ge => Ok(">="),
            TokenType::Not => Ok("!"),
            TokenType::Range => Ok(".."),
            TokenType::True => Ok("true"),
            TokenType::False => Ok("false"),
            TokenType::Mul => Ok("*"),
            //TokenType::Colon => ":",
            _ => return Err(SyntaxErrorKind::UnexpectedToken(self)),
        }
    }
}
