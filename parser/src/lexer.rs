use super::token::Token;

pub struct Lexer {
    src: Vec<char>,
    current: char,
    index: usize,
    eof: bool,
}

impl Lexer {
    pub fn new(src: String) -> Self {
        let src: Vec<char> = src.chars().collect();
        Self {
            current: src[0],
            index: 0,
            src,
            eof: false,
        }
    }

    fn peek(&self, offset: i32) -> char {
        self.src[(self.index as i32 + offset) as usize]
    }

    fn advance(&mut self) {
        if self.index < self.src.len() - 1 {
            self.index += 1;
            self.current = self.src[self.index];
        } else {
            self.eof = true;
        }
    }

    fn advance_with(&mut self, token: Token) -> Token {
        self.advance();
        token
    }

    fn skip_whitespace(&mut self) {
        while (self.current.is_ascii_whitespace()
            || self.current == '\n'
            || self.current == 10 as char)
            && !self.eof
        {
            self.advance();
        }
    }

    fn skip_comment(&mut self) {
        while self.current != '\n' && !self.eof {
            self.advance();
        }
    }

    pub fn next_token(&mut self) -> Token {
        if !self.eof {
            self.skip_whitespace();

            if self.current.is_ascii_digit() {
                return self.parse_number();
            }

            if self.current.is_alphabetic() {
                return self.parse_id();
            }

            match self.current {
                '#' if self.index == 0 || self.peek(-1) == '\n' => self.skip_comment(),
                '$' if self.peek(1).is_alphanumeric() => return self.parse_variable(),
                '"' => return self.parse_string(),
                ')' => return self.advance_with(Token::RightParen),
                '(' => return self.advance_with(Token::LeftParen),
                '}' => return self.advance_with(Token::RightBrace),
                '{' => return self.advance_with(Token::LeftBrace),
                '|' => return self.advance_with(Token::Pipe),
                '<' => return self.advance_with(Token::LessThen),
                '>' => return self.advance_with(Token::GreaterThen),
                '=' => return self.advance_with(Token::Equals),
                ':' => return self.advance_with(Token::Colon),
                ';' => return self.advance_with(Token::SemiColon),
                '+' => return self.advance_with(Token::Plus),
                '-' => return self.advance_with(Token::Minus),
                '*' => return self.advance_with(Token::Mul),
                '^' => return self.advance_with(Token::Pow),
                _ => (),
            };

            return self.parse_until_disallowed();
        }
        Token::EOF
    }

    fn parse_until_disallowed(&mut self) -> Token {
        const DISALLOWED: &str = "\0#$\"(){}|<>=:;+-*^";
        self.advance();
        let mut value = String::new();
        while !DISALLOWED.contains(self.current) && !self.current.is_whitespace() && !self.eof {
            value.push(self.current);
            self.advance();
        }
        Token::Variable(value)
    }

    fn parse_variable(&mut self) -> Token {
        self.advance();
        let mut value = String::new();
        while self.current.is_alphanumeric() || self.current == '_' && !self.eof {
            value.push(self.current);
            self.advance();
        }
        Token::Variable(value)
    }

    fn parse_string(&mut self) -> Token {
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
        Token::String(value)
    }

    fn parse_number(&mut self) -> Token {
        let mut value = String::new();
        while self.current.is_ascii_digit()
            || (self.current == '.' && self.peek(1).is_ascii_digit()) && !self.eof
        {
            value.push(self.current);
            self.advance();
        }
        Token::Number(value.parse().unwrap())
    }

    fn parse_id(&mut self) -> Token {
        let mut value = String::new();
        while self.current.is_alphanumeric() || self.current == '_' && !self.eof {
            value.push(self.current);
            self.advance();
        }
        Token::Identifier(value)
    }
}
