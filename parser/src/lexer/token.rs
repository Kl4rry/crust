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
    Argument(String),
    Variable(String),
    Glob(String),
    String(String),
    Number(f64),
    NewLine,
    Space,
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
