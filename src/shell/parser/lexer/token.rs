use bigdecimal::{num_bigint::BigUint, BigDecimal};
pub mod span;

use std::mem;

use span::Span;

use crate::parser::{
    ast::{
        expr::{
            argument::{ArgumentPart, ArgumentPartKind},
            binop::{BinOp, BinOpKind},
            ExprKind,
        },
        literal::LiteralKind,
        statement::assign_op::{AssignOp, AssignOpKind},
    },
    syntax_error::SyntaxErrorKind,
    Result,
};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub span: Span,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TokenType {
    Symbol(String),
    Float(BigDecimal, String),
    Int(BigUint, String),
    Control,
    DoubleQuote,
    Quote,
    NewLine,
    Space,
    /// &
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
    QuestionMark,

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
                | QuestionMark
                | Dot
                | At
                | LeftBracket
                | LeftParen
                | LeftBrace
                | Quote
                | DoubleQuote
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
                | Lt
                | Not
                | Match
                | NotMatch
                | Symbol(_)
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
    pub fn expect(self, token_type: TokenType) -> Result<Self> {
        if mem::discriminant(&self.token_type) == mem::discriminant(&token_type) {
            Ok(self)
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
            Symbol(text) => Ok(ArgumentPartKind::Bare(text).spanned(self.span)),
            Int(number, _) => Ok(ArgumentPartKind::Int(number.into()).spanned(self.span)),
            Float(number, _) => Ok(ArgumentPartKind::Float(number).spanned(self.span)),
            True => Ok(ArgumentPartKind::Expr(
                ExprKind::Literal(LiteralKind::Bool(true).spanned(self.span)).spanned(self.span),
            )
            .spanned(self.span)),
            False => Ok(ArgumentPartKind::Expr(
                ExprKind::Literal(LiteralKind::Bool(false).spanned(self.span)).spanned(self.span),
            )
            .spanned(self.span)),
            _ => {
                let span = self.span;
                Ok(ArgumentPartKind::Bare(self.try_into_glob_str()?.to_string()).spanned(span))
            }
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
            Gt => Ok(">"),
            Lt => Ok("<"),
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
            QuestionMark => Ok("?"),
            _ => Err(SyntaxErrorKind::UnexpectedToken(self)),
        }
    }

    /// Get binop from token. Will panic if token is not valid binop.
    pub fn to_binop(&self) -> BinOp {
        use TokenType::*;
        match self.token_type {
            Add => BinOpKind::Add,
            Sub => BinOpKind::Sub,
            Mul => BinOpKind::Mul,
            Div => BinOpKind::Div,
            Expo => BinOpKind::Expo,
            Mod => BinOpKind::Mod,
            Eq => BinOpKind::Eq,
            Lt => BinOpKind::Lt,
            Le => BinOpKind::Le,
            Ne => BinOpKind::Ne,
            Ge => BinOpKind::Ge,
            Gt => BinOpKind::Gt,
            And => BinOpKind::And,
            Or => BinOpKind::Or,
            Range => BinOpKind::Range,
            Match => BinOpKind::Match,
            NotMatch => BinOpKind::NotMatch,
            _ => panic!("could not convert token {:?} to binop", self),
        }
        .spanned(self.span)
    }

    /// Get assign op from token. Will panic if token is not valid assign op.
    pub fn to_assign_op(&self) -> AssignOp {
        match self.token_type {
            TokenType::AddAssign => AssignOpKind::Add.spanned(self.span),
            TokenType::SubAssign => AssignOpKind::Sub.spanned(self.span),
            TokenType::MulAssign => AssignOpKind::Mul.spanned(self.span),
            TokenType::DivAssign => AssignOpKind::Div.spanned(self.span),
            TokenType::ExpoAssign => AssignOpKind::Expo.spanned(self.span),
            TokenType::ModAssign => AssignOpKind::Mod.spanned(self.span),
            _ => panic!("could not convert token {:?} to assign op", self),
        }
    }
}

pub fn is_valid_identifier(ident: &str) -> bool {
    match ident.chars().next() {
        Some(c) => {
            if c != '_' && !c.is_ascii_alphabetic() {
                return false;
            }
        }
        None => return false,
    }
    ident.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}
