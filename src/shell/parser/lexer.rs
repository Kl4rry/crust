pub mod token;

use memchr::memchr;
use token::{span::Span, Token, TokenType};

pub const ESCAPES: &[u8] = b"nt0rs\\";
pub const REPLACEMENTS: &[u8] = b"\n\t\0\r \\";

#[inline(always)]
pub fn escape_char(c: u8) -> u8 {
    memchr(c, ESCAPES).map(|i| REPLACEMENTS[i]).unwrap_or(c)
}

pub struct Lexer {
    src: String,
    current: u8,
    index: usize,
    eof: bool,
}

impl Lexer {
    pub fn new(src: String) -> Self {
        let (current, eof) = if src.is_empty() {
            // this null byte should NEVER be read as the eof flag is set to true
            (b'\0', true)
        } else {
            (src.as_bytes()[0], false)
        };

        Self {
            current,
            index: 0,
            src,
            eof,
        }
    }

    #[inline(always)]
    pub fn src(&self) -> &str {
        &self.src
    }

    #[inline(always)]
    fn peek(&self, offset: i32) -> u8 {
        self.src.as_bytes()[(self.index as i32 + offset) as usize]
    }

    #[inline(always)]
    fn advance(&mut self) {
        if (self.index < self.src.len() - 1) && !self.eof {
            self.index += 1;
            self.current = self.src.as_bytes()[self.index];
        } else if !self.eof {
            self.index += 1;
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
        let mut value = String::new();
        const DISALLOWED: &[u8] = b"\0#$\"\'(){}[]|;&,";
        while !DISALLOWED.contains(&self.current)
            && !self.current.is_ascii_whitespace()
            && !self.eof
        {
            if self.current == b'\\' {
                self.advance();
                let escape = escape_char(self.current);
                unsafe { value.as_mut_vec().push(escape) };
                self.advance();
            } else {
                unsafe { value.as_mut_vec().push(self.current) };
                self.advance();
            }
        }
        let end = self.index;
        let span = Span::new(start, end);

        let token_type = match value.as_str() {
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "loop" => TokenType::Loop,
            "for" => TokenType::For,
            "in" => TokenType::In,
            "break" => TokenType::Break,
            "return" => TokenType::Return,
            "continue" => TokenType::Continue,
            "fn" => TokenType::Fn,
            "true" => TokenType::True,
            "false" => TokenType::False,
            "let" => TokenType::Let,
            "export" => TokenType::Export,
            _ => {
                return Token {
                    token_type: TokenType::Symbol(value),
                    span,
                }
            }
        };

        Token { token_type, span }
    }

    fn parse_variable(&mut self) -> Token {
        let start = self.index;
        self.advance();
        while (self.current.is_ascii_alphanumeric() || self.current == b'_') && !self.eof {
            self.advance();
        }
        let end = self.index;
        let value = self.src[start + 1..end].to_string();

        Token {
            token_type: TokenType::Variable(value),
            span: Span::new(start, end),
        }
    }

    fn parse_string(&mut self) -> Token {
        let start = self.index;
        self.advance();
        let mut value = String::new();
        while !self.eof {
            if self.current == b'\'' {
                self.advance();
                break;
            } else {
                unsafe { value.as_mut_vec().push(self.current) };
                self.advance();
            }
        }
        let end = self.index;

        Token {
            token_type: TokenType::String(value),
            span: Span::new(start, end),
        }
    }

    fn parse_number(&mut self) -> Token {
        let start = self.index;
        let mut float = false;
        let mut value = Vec::new();
        while (self.current.is_ascii_digit()
            || self.current == b'_'
            || (self.current == b'.'
                && self.index + 1 < self.src.len()
                && self.peek(1).is_ascii_digit()))
            && !self.eof
        {
            if self.current == b'.' {
                if float {
                    let end = self.index;
                    return Token {
                        token_type: TokenType::Symbol(self.src[start..end].to_string()),
                        span: Span::new(start, end),
                    };
                }
                float = true;
            }
            if self.current != b'_' {
                value.push(self.current);
            }
            self.advance();
        }
        let end = self.index;
        let value = String::from_utf8(value).unwrap();
        let string = self.src[start..end].to_string();

        if float {
            Token {
                token_type: TokenType::Float(value.parse().unwrap(), string),
                span: Span::new(start, end),
            }
        } else {
            Token {
                token_type: TokenType::Int(value.parse().unwrap(), string),
                span: Span::new(start, end),
            }
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
                    self.parse_newline()?
                }
                b'$' => {
                    if self.index + 1 < self.src.len() && self.peek(1).is_ascii_alphanumeric() {
                        self.parse_variable()
                    } else {
                        self.advance_with(TokenType::Dollar, 1)
                    }
                }

                b'.' if self.index + 1 < self.src.len() && self.peek(1) == b'.' => {
                    self.advance_with(TokenType::Range, 2)
                }
                b',' => self.advance_with(TokenType::Comma, 1),
                b'|' => {
                    if self.index + 1 < self.src.len() && self.peek(1) == b'|' {
                        self.advance_with(TokenType::Or, 2)
                    } else {
                        self.advance_with(TokenType::Pipe, 1)
                    }
                }
                b'&' => {
                    if self.index + 1 < self.src.len() && self.peek(1) == b'&' {
                        self.advance_with(TokenType::And, 2)
                    } else {
                        self.advance_with(TokenType::Exec, 1)
                    }
                }
                b'\'' => self.parse_string(),
                b'"' => self.advance_with(TokenType::Quote, 1),
                b')' => self.advance_with(TokenType::RightParen, 1),
                b'(' => self.advance_with(TokenType::LeftParen, 1),
                b'}' => self.advance_with(TokenType::RightBrace, 1),
                b'{' => self.advance_with(TokenType::LeftBrace, 1),
                b']' => self.advance_with(TokenType::RightBracket, 1),
                b'[' => self.advance_with(TokenType::LeftBracket, 1),
                b':' => self.advance_with(TokenType::Colon, 1),
                b';' => self.advance_with(TokenType::SemiColon, 1),
                // binary operators
                b'=' => {
                    if self.index + 1 < self.src.len() && self.peek(1) == b'=' {
                        self.advance_with(TokenType::Eq, 2)
                    } else {
                        self.advance_with(TokenType::Assignment, 1)
                    }
                }
                b'+' => {
                    if self.index + 1 < self.src.len() && self.peek(1) == b'=' {
                        self.advance_with(TokenType::AddAssign, 2)
                    } else {
                        self.advance_with(TokenType::Add, 1)
                    }
                }
                b'-' => {
                    if self.index + 1 < self.src.len() && self.peek(1) == b'=' {
                        self.advance_with(TokenType::SubAssign, 2)
                    } else {
                        self.advance_with(TokenType::Sub, 1)
                    }
                }
                b'/' => {
                    if self.index + 1 < self.src.len() && self.peek(1) == b'=' {
                        self.advance_with(TokenType::DivAssign, 2)
                    } else {
                        self.advance_with(TokenType::Div, 1)
                    }
                }
                b'%' => {
                    if self.index + 1 < self.src.len() && self.peek(1) == b'=' {
                        self.advance_with(TokenType::ModAssign, 2)
                    } else {
                        self.advance_with(TokenType::Mod, 1)
                    }
                }
                b'*' => {
                    if self.index + 1 < self.src.len() && self.peek(1) == b'*' {
                        if self.index + 2 < self.src.len() && self.peek(2) == b'=' {
                            self.advance_with(TokenType::ExpoAssign, 3)
                        } else {
                            self.advance_with(TokenType::Expo, 2)
                        }
                    } else if self.index + 1 < self.src.len() && self.peek(1) == b'=' {
                        self.advance_with(TokenType::MulAssign, 2)
                    } else {
                        self.advance_with(TokenType::Mul, 1)
                    }
                }

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
