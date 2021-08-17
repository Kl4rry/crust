pub mod span;
use smallstr::SmallString;

use crate::error::SyntaxError;
use span::Span;

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
    Colon,
    SemiColon,
    QuestionMark,
    // Binary operators
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
}

impl TokenType {
    pub fn is_space(&self) -> bool {
        matches!(*self, Self::Space)
    }

    pub fn is_unop(&self) -> bool {
        matches!(
            *self,
            Self::Not
                | Self::Sub
        )
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
        )
    }
}

impl Token {
    pub fn is_space(&self) -> bool {
        self.token_type.is_space()
    }

    pub fn is_binop(&self) -> bool {
        self.token_type.is_binop()
    }

    pub fn try_into_arg(self) -> Result<SmallString<[u8; 10]>, SyntaxError> {
        Ok(SmallString::from(match self.token_type {
            TokenType::Assignment => "=",
            TokenType::Colon => ":",
            TokenType::QuestionMark => "?",
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
            _ => return Err(SyntaxError::UnexpectedToken(self)),
        }))
    }
}
