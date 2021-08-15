pub mod token;
use token::{span::Span, Token, TokenType};

pub struct Lexer {
    src: String,
    current: u8,
    index: usize,
    eof: bool,
}

impl Lexer {
    pub fn new(src: String) -> Self {
        Self {
            current: src.as_bytes()[0],
            index: 0,
            src,
            eof: false,
        }
    }

    #[inline(always)]
    fn peek(&self, offset: i32) -> u8 {
        self.src.as_bytes()[(self.index as i32 + offset) as usize]
    }

    #[inline(always)]
    fn advance(&mut self) {
        if self.index < self.src.len() - 1 {
            self.index += 1;
            self.current = self.src.as_bytes()[self.index];
        } else {
            self.eof = true;
        }
    }

    #[inline(always)]
    fn advance_with(&mut self, token_type: TokenType, length: usize) -> Token {
        assert!(length > 0);

        let start = self.index;
        for _ in 0..length {
            self.advance();
        }
        let end = self.index;

        Token {
            token_type,
            span: Span::new(start, end),
        }
    }

    fn parse_whitespace(&mut self) -> Option<Token> {
        let mut advanced = false;
        let start = self.index;
        while self.current.is_ascii_whitespace() && self.current != b'\n' && !self.eof {
            advanced = true;
            self.advance();
        }
        let end = self.index;

        if advanced {
            Some(Token {
                token_type: TokenType::Space,
                span: Span::new(start, end),
            })
        } else {
            None
        }
    }

    fn parse_newline(&mut self) -> Option<Token> {
        if self.current == b'\n' && !self.eof {
            let token = Some(Token {
                token_type: TokenType::NewLine,
                span: Span::new(self.index, self.index),
            });
            self.advance();
            token
        } else {
            None
        }
    }

    fn skip_comment(&mut self) {
        while (self.current != b'\n') && !self.eof {
            self.advance();
        }
    }

    fn parse_arg(&mut self) -> Token {
        let start = self.index;
        const DISALLOWED: &[u8] = b"\0#$\"(){}|;&";
        while !DISALLOWED.contains(&self.current)
            && !self.current.is_ascii_whitespace()
            && !self.eof
        {
            self.advance();
        }
        let end = self.index;
        let value = self.src[start..end].to_string();

        Token {
            token_type: TokenType::Symbol(value),
            span: Span::new(start, end),
        }
    }

    fn parse_variable(&mut self) -> Token {
        let start = self.index;
        self.advance();
        while self.current.is_ascii_alphanumeric() || self.current == b'_' && !self.eof {
            self.advance();
        }
        let end = self.index;
        let value = self.src[start + 1..end].to_string();

        Token {
            token_type: TokenType::Variable(value),
            span: Span::new(start, end),
        }
    }

    fn parse_expand_string(&mut self) -> Token {
        let start = self.index;
        self.advance();
        while !self.eof {
            if self.current == b'"' {
                self.advance();
                break;
            }
            self.advance();
        }
        let end = self.index;
        let value = self.src[start + 1..end - 1].to_string();

        Token {
            token_type: TokenType::ExpandString(value),
            span: Span::new(start, end),
        }
    }

    fn parse_string(&mut self) -> Token {
        let start = self.index;
        self.advance();
        while !self.eof {
            if self.current == b'\'' {
                self.advance();
                break;
            }
            self.advance();
        }
        let end = self.index;
        let value = self.src[start + 1..end - 1].to_string();

        Token {
            token_type: TokenType::String(value),
            span: Span::new(start, end),
        }
    }

    fn parse_number(&mut self) -> Token {
        let start = self.index;
        while (self.current.is_ascii_digit()
            || (self.current == b'.' && self.peek(1).is_ascii_digit()))
            && !self.eof
        {
            self.advance();
        }
        let end = self.index;
        let value = &self.src[start..end].to_string();

        Token {
            token_type: TokenType::Number(value.parse().unwrap()),
            span: Span::new(start, end),
        }
    }
}

impl Iterator for Lexer {
    type Item = Token;
    #[inline]
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
                b'#' => {
                    self.skip_comment();
                    if let Some(token) = self.parse_newline() {
                        token
                    } else {
                        return None;
                    }
                }
                b'$' if self.peek(1).is_ascii_alphabetic() => self.parse_variable(),
                b'|' => self.advance_with(TokenType::Pipe, 1),
                b'"' => self.parse_expand_string(),
                b'\'' => self.parse_string(),
                b')' => self.advance_with(TokenType::RightParen, 1),
                b'(' => self.advance_with(TokenType::LeftParen, 1),
                b'}' => self.advance_with(TokenType::RightBrace, 1),
                b'{' => self.advance_with(TokenType::LeftBrace, 1),
                b':' => self.advance_with(TokenType::Colon, 1),
                b';' => self.advance_with(TokenType::SemiColon, 1),
                b'?' => self.advance_with(TokenType::QuestionMark, 1),
                // binary operators
                b'=' => {
                    if self.index + 1 < self.src.len() && self.peek(1) == b'=' {
                        self.advance_with(TokenType::Equality, 2)
                    } else {
                        self.advance_with(TokenType::Assignment, 1)
                    }
                }
                b'&' => {
                    if self.index + 1 < self.src.len() && self.peek(1) == b'=' {
                        self.advance_with(TokenType::And, 2)
                    } else {
                        self.advance_with(TokenType::Exec, 1)
                    }
                }
                b'+' => self.advance_with(TokenType::Add, 1),
                b'-' => self.advance_with(TokenType::Sub, 1),
                b'*' => self.advance_with(TokenType::Mul, 1),
                b'/' => self.advance_with(TokenType::Div, 1),
                b'%' => self.advance_with(TokenType::Mod, 1),

                b'<' => {
                    if self.index + 1 < self.src.len() && self.peek(1) == b'=' {
                        self.advance_with(TokenType::Le, 2)
                    } else {
                        self.advance_with(TokenType::Lt, 1)
                    }
                }
                b'>' => {
                    if self.index + 1 < self.src.len() && self.peek(1) == b'=' {
                        self.advance_with(TokenType::Ge, 2)
                    } else {
                        self.advance_with(TokenType::Gt, 1)
                    }
                }
                b'!' => {
                    if self.index + 1 < self.src.len() && self.peek(1) == b'=' {
                        self.advance_with(TokenType::Ne, 2)
                    } else {
                        self.advance_with(TokenType::Not, 1)
                    }
                }
                _ => self.parse_arg(),
            };
            return Some(token);
        }
        None
    }
}
