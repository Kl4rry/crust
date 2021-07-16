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

    #[inline(always)]
    fn advance_with(&mut self, token: Token) -> Token {
        self.advance();
        token
    }

    fn parse_whitespace(&mut self) -> Option<Token> {
        let mut new_line = false;
        let mut advanced = false;
        while self.current.is_ascii_whitespace() && !self.eof {
            advanced = true;
            if self.current == '\n' || self.current == '\r' {
                new_line = true;
            }
            self.advance();
        }

        if advanced {
            if new_line {
                Some(Token::NewLine)
            } else {
                Some(Token::Space)
            }
        } else {
            None
        }
    }

    fn skip_comment(&mut self) {
        while (self.current != '\n' && self.current != '\r') && !self.eof {
            self.advance();
        }
    }

    pub fn next_token(&mut self) -> Token {
        if !self.eof {
            if let Some(token) = self.parse_whitespace() {
                return token;
            }

            if self.current.is_ascii_digit() {
                return self.parse_number();
            }

            match self.current {
                '#' => {
                    self.skip_comment();
                    return Token::NewLine;
                }
                '$' if self.peek(1).is_alphabetic() => return self.parse_variable(),
                '=' if self.peek(1) == '=' && self.index + 1 < self.src.len() => {
                    self.advance();
                    return self.advance_with(Token::Equality);
                }
                '"' => return self.parse_string(),
                ')' => return self.advance_with(Token::RightParen),
                '(' => return self.advance_with(Token::LeftParen),
                '}' => return self.advance_with(Token::RightBrace),
                '{' => return self.advance_with(Token::LeftBrace),
                '|' => return self.advance_with(Token::Pipe),
                '<' => return self.advance_with(Token::LessThen),
                '>' => return self.advance_with(Token::GreaterThen),
                ':' => return self.advance_with(Token::Colon),
                ';' => return self.advance_with(Token::SemiColon),
                _ => (),
            };
            return self.parse_arg();
        }
        Token::EOF
    }

    fn parse_arg(&mut self) -> Token {
        const DISALLOWED: &str = "\0#$\"(){}|;";
        let mut value = String::new();
        while !DISALLOWED.contains(self.current) && !self.current.is_whitespace() && !self.eof {
            value.push(self.current);
            self.advance();
        }
        if value.contains('*') {
            Token::Glob(value)
        } else {
            Token::Argument(value)
        }
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
}
