pub mod span;
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
    Equality,
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
        match *self {
            Self::Space => true,
            _ => false,
        }
    }

    pub fn is_binop(&self) -> bool {
        match *self {
            Self::Add
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
            | Self::Or => true,
            _ => false,
        }
    }
}

impl Token {
    pub fn is_space(&self) -> bool {
        self.token_type.is_space()
    }
}
