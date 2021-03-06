use bigdecimal::{num_bigint::BigUint, BigDecimal};
pub mod span;

use std::{convert::TryInto, mem};

use span::Span;

use crate::parser::{
    ast::{
        expr::{argument::ArgumentPart, binop::BinOp, Expr},
        literal::Literal,
        statement::assign_op::AssignOp,
    },
    syntax_error::SyntaxErrorKind,
    Result,
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
    Float(BigDecimal, String),
    Int(BigUint, String),
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
    At,
    Colon,
    SemiColon,
    Dot,

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
    /// The =~ operator
    Match,
    /// The !~ operator
    NotMatch,
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
    Let,
    Export,
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
                | Self::Match
                | Self::NotMatch
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
    pub fn is_valid_argpart(&self) -> bool {
        use TokenType::*;
        matches!(
            *self,
            Dollar
                | Dot
                | At
                | LeftBracket
                | LeftParen
                | Quote
                | AddAssign
                | SubAssign
                | MulAssign
                | DivAssign
                | ExpoAssign
                | ModAssign
                | Assignment
                | Colon
                | Range
                | Add
                | Sub
                | Mul
                | Div
                | Expo
                | Mod
                | Eq
                | Le
                | Ne
                | Ge
                | Not
                | Match
                | NotMatch
                | Symbol(_)
                | Variable(_)
                | String(_)
                | Float(_, _)
                | Int(_, _)
                | True
                | False
                | Let
                | Export
        )
    }

    pub fn is_keyword(&self) -> bool {
        use TokenType::*;
        matches!(
            *self,
            If | Else | While | Loop | For | In | Break | Return | Continue | Fn | Let | Export
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

    pub fn is_valid_argpart(&self) -> bool {
        self.token_type.is_valid_argpart()
    }

    pub fn try_into_argpart(self) -> Result<ArgumentPart> {
        use TokenType::*;
        match self.token_type {
            String(text) => Ok(ArgumentPart::Quoted(text)),
            Symbol(text) => Ok(ArgumentPart::Bare(text)),
            Variable(_) => Ok(ArgumentPart::Variable(self.try_into()?)),
            Int(number, _) => Ok(ArgumentPart::Int(number.into())),
            Float(number, _) => Ok(ArgumentPart::Float(number)),
            True => Ok(ArgumentPart::Expr(Expr::Literal(Literal::Bool(true)))),
            _ => return Ok(ArgumentPart::Bare(self.try_into_glob_str()?.to_string())),
        }
    }

    pub fn try_into_glob_str(self) -> Result<&'static str> {
        use TokenType::*;
        match self.token_type {
            AddAssign => Ok("+="),
            SubAssign => Ok("-="),
            MulAssign => Ok("*="),
            DivAssign => Ok("/="),
            ExpoAssign => Ok("**="),
            ModAssign => Ok("%="),
            Match => Ok("=~"),
            NotMatch => Ok("!~"),
            Assignment => Ok("="),
            RightBracket => Ok("]"),
            LeftBracket => Ok("["),
            Add => Ok("+"),
            Sub => Ok("-"),
            Div => Ok("/"),
            Expo => Ok("**"),
            Mod => Ok("%"),
            Eq => Ok("=="),
            Le => Ok("<="),
            Ne => Ok("-"),
            Ge => Ok(">="),
            Not => Ok("!"),
            Range => Ok(".."),
            True => Ok("true"),
            False => Ok("false"),
            Mul => Ok("*"),
            Colon => Ok(":"),
            Let => Ok("let"),
            Export => Ok("export"),
            At => Ok("@"),
            Dot => Ok("."),
            _ => Err(SyntaxErrorKind::UnexpectedToken(self)),
        }
    }

    /// Get binop from token. Will panic if token is not valid binop.
    pub fn to_binop(&self) -> BinOp {
        use TokenType::*;
        match self.token_type {
            Add => BinOp::Add,
            Sub => BinOp::Sub,
            Mul => BinOp::Mul,
            Div => BinOp::Div,
            Expo => BinOp::Expo,
            Mod => BinOp::Mod,
            Eq => BinOp::Eq,
            Lt => BinOp::Lt,
            Le => BinOp::Le,
            Ne => BinOp::Ne,
            Ge => BinOp::Ge,
            Gt => BinOp::Gt,
            And => BinOp::And,
            Or => BinOp::Or,
            Range => BinOp::Range,
            Match => BinOp::Match,
            NotMatch => BinOp::NotMatch,
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
