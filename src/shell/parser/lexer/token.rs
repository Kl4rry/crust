pub mod span;

use std::{convert::TryInto, mem};

use span::Span;

use crate::{
    parser::{
        ast::{
            expr::{argument::Identifier, binop::BinOp},
            statement::assign_op::AssignOp,
        },
        syntax_error::SyntaxErrorKind,
        Result,
    },
    shell::value::Type,
};

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
    Cast(Type),
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
    Colon,
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

    // Assign operators
    /// The += operator
    AddAssign,
    /// The -= operator
    SubAssign,
    /// The *= operator
    MulAssign,
    /// The /= operator
    DivAssign,
    /// The **= operator
    ExpoAssign,
    /// The %= operator
    ModAssign,

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

    pub fn is_assign_op(&self) -> bool {
        matches!(
            *self,
            Self::AddAssign
                | Self::SubAssign
                | Self::MulAssign
                | Self::DivAssign
                | Self::ExpoAssign
                | Self::ModAssign
        )
    }

    // check if token can be passed as a string arg to a call
    pub fn is_valid_id(&self) -> bool {
        matches!(
            *self,
            TokenType::Dollar
                | TokenType::Quote
                | TokenType::AddAssign
                | TokenType::SubAssign
                | TokenType::MulAssign
                | TokenType::DivAssign
                | TokenType::ExpoAssign
                | TokenType::ModAssign
                | TokenType::Assignment
                | TokenType::Colon
                | TokenType::RightBracket
                | TokenType::LeftBracket
                | TokenType::Range
                | TokenType::Add
                | TokenType::Sub
                | TokenType::Mul
                | TokenType::Div
                | TokenType::Expo
                | TokenType::Mod
                | TokenType::Eq
                | TokenType::Le
                | TokenType::Ne
                | TokenType::Ge
                | TokenType::Not
                | TokenType::Symbol(_)
                | TokenType::Variable(_)
                | TokenType::String(_)
                | TokenType::Float(_, _)
                | TokenType::Int(_, _)
                | TokenType::True
                | TokenType::False
                | TokenType::Cast(_)
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
            TokenType::Symbol(text) => Ok(Identifier::Bare(text)),
            TokenType::Variable(_) => Ok(Identifier::Variable(self.try_into()?)),
            TokenType::Int(_, text) => Ok(Identifier::Bare(text)),
            TokenType::Float(_, text) => Ok(Identifier::Bare(text)),
            _ => return Ok(Identifier::Bare(self.try_into_glob_str()?.to_string())),
        }
    }

    pub fn try_into_glob_str(self) -> Result<&'static str> {
        match self.token_type {
            TokenType::AddAssign => Ok("+="),
            TokenType::SubAssign => Ok("-="),
            TokenType::MulAssign => Ok("*="),
            TokenType::DivAssign => Ok("/="),
            TokenType::ExpoAssign => Ok("**="),
            TokenType::ModAssign => Ok("%="),
            TokenType::Assignment => Ok("="),
            TokenType::RightBracket => Ok("]"),
            TokenType::LeftBracket => Ok("["),
            TokenType::Add => Ok("+"),
            TokenType::Sub => Ok("-"),
            TokenType::Div => Ok("/"),
            TokenType::Expo => Ok("**"),
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
            TokenType::Colon => Ok(":"),
            TokenType::Cast(t) => Ok(t.as_str()),
            _ => Err(SyntaxErrorKind::UnexpectedToken(self)),
        }
    }

    /// Get binop from token. Will panic if token is not valid binop.
    pub fn to_binop(&self) -> BinOp {
        match self.token_type {
            TokenType::Add => BinOp::Add,
            TokenType::Sub => BinOp::Sub,
            TokenType::Mul => BinOp::Mul,
            TokenType::Div => BinOp::Div,
            TokenType::Expo => BinOp::Expo,
            TokenType::Mod => BinOp::Mod,
            TokenType::Eq => BinOp::Eq,
            TokenType::Lt => BinOp::Lt,
            TokenType::Le => BinOp::Le,
            TokenType::Ne => BinOp::Ne,
            TokenType::Ge => BinOp::Ge,
            TokenType::Gt => BinOp::Gt,
            TokenType::And => BinOp::And,
            TokenType::Or => BinOp::Or,
            TokenType::Range => BinOp::Range,
            _ => panic!("could not convert token {:?} to binop", self),
        }
    }

    /// Get assign op from token. Will panic if token is not valid assign op.
    pub fn to_assign_op(&self) -> AssignOp {
        match self.token_type {
            TokenType::AddAssign => AssignOp::Add,
            TokenType::SubAssign => AssignOp::Sub,
            TokenType::MulAssign => AssignOp::Mul,
            TokenType::DivAssign => AssignOp::Div,
            TokenType::ExpoAssign => AssignOp::Expo,
            TokenType::ModAssign => AssignOp::Mod,
            _ => panic!("could not convert token {:?} to assign op", self),
        }
    }
}
