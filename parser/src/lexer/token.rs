#[derive(PartialEq, Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub c_start: usize,
    pub c_end: usize,
    pub r_start: usize,
    pub r_end: usize,
}

#[derive(PartialEq, Debug, Clone)]
pub enum TokenType {
    Symbol(String),
    Variable(String),
    String(String),
    ExpandString(String),
    Number(f64),
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
}

impl TokenType {
    pub fn is_space(&self) -> bool {
        match *self {
            Self::Space => true,
            _ => false,
        }
    }
}

impl Token {
    pub fn is_space(&self) -> bool {
        self.token_type.is_space()
    }
}
