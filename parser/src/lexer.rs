use super::token::{Token, TokenType};

pub struct Lexer {
    src: Vec<char>,
    current: char,
    index: usize,
    eof: bool,
    row: usize,
    column: usize,
}

impl Lexer {
    pub fn new(src: String) -> Self {
        let src: Vec<char> = src.chars().collect();
        Self {
            current: src[0],
            index: 0,
            src,
            eof: false,
            row: 0,
            column: 0,

        }
    }

    #[inline(always)]
    fn peek(&self, offset: i32) -> char {
        self.src[(self.index as i32 + offset) as usize]
    }

    #[inline(always)]
    fn advance(&mut self) {
        if self.index < self.src.len() - 1 {
            self.index += 1;
            self.current = self.src[self.index];

            if self.index > 0 {
                if self.peek(-1) == '\n' {
                    self.row += 1;
                    self.column = 0;
                } else {
                    self.column += 1;
                }
            }
        } else {
            self.eof = true;
        }
    }

    #[inline(always)]
    fn advance_with(&mut self, token_type: TokenType, length: usize) -> Token {
        assert!(length > 0);

        let c_start = self.column;
        let r_start = self.row;
        for _ in 0..length {
            self.advance();
        }

        Token {
            token_type,
            c_start,
            r_start,
            c_end: self.column,
            r_end: self.row,
        }
    }

    fn parse_whitespace(&mut self) -> Option<Token> {
        let mut advanced = false;
        let c_start = self.column;
        let r_start = self.row;
        while self.current.is_ascii_whitespace() && self.current != '\n' && !self.eof {
            advanced = true;
            self.advance();
        }

        if advanced {
            Some(Token {
                token_type: TokenType::Space,
                c_start,
                r_start,
                c_end: self.column,
                r_end: self.row,
            })
        } else {
            None
        }
    }

    fn parse_newline(&mut self) -> Option<Token> {
        let c_start = self.column;
        let r_start = self.row;
        if self.current == '\n' && !self.eof {
            let token = Some(Token {
                token_type: TokenType::NewLine,
                c_start,
                r_start,
                c_end: self.column,
                r_end: self.row,
            });
            self.advance();
            token
        } else {
            None
        }
    }

    fn skip_comment(&mut self) {
        while (self.current != '\n') && !self.eof {
            self.advance();
        }
    }

    fn parse_arg(&mut self) -> Token {
        let c_start = self.column;
        let r_start = self.row;

        const DISALLOWED: &str = "\0#$\"(){}|;";
        let mut value = String::new();
        while !DISALLOWED.contains(self.current) && !self.current.is_whitespace() && !self.eof {
            value.push(self.current);
            self.advance();
        }
        if value.contains('*') {
            Token {
                token_type: TokenType::Glob(value),
                c_start,
                r_start,
                c_end: self.column,
                r_end: self.row,
            }
        } else {
            Token {
                token_type: TokenType::Argument(value),
                c_start,
                r_start,
                c_end: self.column,
                r_end: self.row,
            }
        }
    }

    fn parse_variable(&mut self) -> Token {
        let c_start = self.column;
        let r_start = self.row;

        self.advance();
        let mut value = String::new();
        while self.current.is_alphanumeric() || self.current == '_' && !self.eof {
            value.push(self.current);
            self.advance();
        }

        Token {
            token_type: TokenType::Variable(value),
            c_start,
            r_start,
            c_end: self.column,
            r_end: self.row,
        }
    }

    fn parse_string(&mut self) -> Token {
        let c_start = self.column;
        let r_start = self.row;

        self.advance();
        let mut value = String::new();
        while !self.eof {
            if self.current == '"' {
                self.advance();
                break;
            }
            value.push(self.current);
            self.advance();
        }

        Token {
            token_type: TokenType::String(value),
            c_start,
            r_start,
            c_end: self.column,
            r_end: self.row,
        }
    }

    fn parse_number(&mut self) -> Token {
        let c_start = self.column;
        let r_start = self.row;

        let mut value = String::new();
        while self.current.is_ascii_digit()
            || (self.current == '.' && self.peek(1).is_ascii_digit()) && !self.eof
        {
            value.push(self.current);
            self.advance();
        }

        Token {
            token_type: TokenType::Number(value.parse().unwrap()),
            c_start,
            r_start,
            c_end: self.column,
            r_end: self.row,
        }
    }
}

impl Iterator for Lexer {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        if !self.eof {
            if let Some(token) = self.parse_whitespace() {
                return Some(token);
            }

            if let Some(token) = self.parse_newline() {
                return Some(token);
            }

            if self.current.is_ascii_digit() {
                return Some(self.parse_number());
            }

            let token = match self.current {
                '#' => {
                    self.skip_comment();
                    self.parse_newline().unwrap()
                }
                '$' if self.peek(1).is_alphabetic() => self.parse_variable(),
                '=' if self.peek(1) == '=' && self.index + 1 < self.src.len() => {
                    self.advance_with(TokenType::Equality, 2)
                }
                '"' => self.parse_string(),
                ')' => self.advance_with(TokenType::RightParen, 1),
                '(' => self.advance_with(TokenType::LeftParen, 1),
                '}' => self.advance_with(TokenType::RightBrace, 1),
                '{' => self.advance_with(TokenType::LeftBrace, 1),
                '|' => self.advance_with(TokenType::Pipe, 1),
                '<' => self.advance_with(TokenType::LessThen, 1),
                '>' => self.advance_with(TokenType::GreaterThen, 1),
                ':' => self.advance_with(TokenType::Colon, 1),
                ';' => self.advance_with(TokenType::SemiColon, 1),
                '?' => self.advance_with(TokenType::QuestionMark, 1),
                _ => self.parse_arg(),
            };
            return Some(token);
        }
        None
    }
}
