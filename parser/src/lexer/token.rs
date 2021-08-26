pub mod span;

use std::mem;

use smallstr::SmallString;
use span::Span;

use crate::{error::SyntaxError, Result, Small};

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
    ExpandString(String),
    Float(f64, String),
    Int(u128, String),
    NewLine,
    Space,
    Exec,
    Assignment,
    Pipe,
    LessThen,
    GreaterThen,
    RightBrace,
    LeftBrace,
    RightParen,
    LeftParen,
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

    pub fn is_valid_arg(&self) -> bool {
        matches!(
            *self,
            TokenType::Assignment |
            //TokenType::Colon |
            TokenType::Range |
            TokenType::Add |
            TokenType::Sub |
            TokenType::Mul |
            TokenType::Div |
            TokenType::Expo |
            TokenType::Mod |
            TokenType::Eq  |
            TokenType::Lt |
            TokenType::Le  |
            TokenType::Ne |
            TokenType::Ge  |
            TokenType::Gt |
            TokenType::Not |
            TokenType::Symbol(_) |
            TokenType::Variable(_) |
            TokenType::String(_) |
            TokenType::ExpandString(_) |
            TokenType::Float(_, _) |
            TokenType::Int(_, _)
        )
    }
}

impl Token {
    pub fn expect(self, token_type: TokenType) -> Result<()> {
        if mem::discriminant(&self.token_type) == mem::discriminant(&token_type) {
            Ok(())
        } else {
            Err(SyntaxError::UnexpectedToken(self))
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

    pub fn is_valid_arg(&self) -> bool {
        self.token_type.is_valid_arg()
    }

    pub fn try_into_arg(self) -> Result<Small> {
        Ok(SmallString::from(match self.token_type {
            TokenType::Assignment => "=",
            //TokenType::Colon => ":",
            TokenType::Add => "+",
            TokenType::Sub => "-",
            TokenType::Mul => "*",
            TokenType::Div => "/",
            TokenType::Expo => "^",
            TokenType::Mod => "%",
            TokenType::Eq => "==",
            TokenType::Lt => "<",
            TokenType::Le => "<=",
            TokenType::Ne => "-",
            TokenType::Ge => ">=",
            TokenType::Gt => ">",
            TokenType::Not => "!",
            TokenType::Range => "..",
            _ => return Err(SyntaxError::UnexpectedToken(self)),
        }))
    }
}
