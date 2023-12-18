pub mod token;

use std::sync::Arc;

use bigdecimal::{num_bigint::BigUint, BigDecimal};
use memchr::memchr;
use token::{span::Span, Token, TokenType};

use super::source::Source;

const LINE_ENDINGS: [&str; 8] = [
    "\u{000D}\u{000A}", // CarriageReturn followed by LineFeed
    "\u{000A}",         // U+000A -- LineFeed
    "\u{000B}",         // U+000B -- VerticalTab
    "\u{000C}",         // U+000C -- FormFeed
    "\u{000D}",         // U+000D -- CarriageReturn
    "\u{0085}",         // U+0085 -- NextLine
    "\u{2028}",         // U+2028 -- Line Separator
    "\u{2029}",         // U+2029 -- ParagraphSeparator
];

fn is_word_break(b: u8) -> bool {
    const DISALLOWED: &[u8] = b"\0#$\"\'(){}[]|;&,.:\\/=";
    DISALLOWED.contains(&b)
        || b.is_ascii_whitespace()
        || LINE_ENDINGS.iter().any(|le| le.as_bytes()[0] == b)
}

pub struct Lexer {
    src: Arc<Source>,
    current: u8,
    index: usize,
    eof: bool,
}

impl Lexer {
    pub fn new(src: Arc<Source>) -> Self {
        let (current, eof) = if src.code.is_empty() {
            // this null byte should NEVER be read as the eof flag is set to true
            (b'\0', true)
        } else {
            (src.code.as_bytes()[0], false)
        };

        Self {
            current,
            index: 0,
            src,
            eof,
        }
    }

    pub fn named_source(&self) -> Arc<Source> {
        self.src.clone()
    }

    #[inline(always)]
    pub fn src(&self) -> &str {
        &self.src.code
    }

    #[inline(always)]
    fn peek(&self, offset: i32) -> Option<u8> {
        self.src()
            .as_bytes()
            .get((self.index as i32 + offset) as usize)
            .copied()
    }

    #[inline(always)]
    fn advance(&mut self) {
        if (self.index < self.src().len() - 1) && !self.eof {
            self.index += 1;
            self.current = self.src().as_bytes()[self.index];
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
        while self.current.is_ascii_whitespace()
            && LINE_ENDINGS
                .iter()
                .all(|byte| byte.as_bytes()[0] != self.current)
            && !self.eof
        {
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
        let start = self.index;
        let mut token = None;
        let ending: &str = LINE_ENDINGS
            .iter()
            .find(|s| s.as_bytes()[0] == self.current)?;
        for byte in ending.as_bytes() {
            if self.eof {
                return token;
            }

            token = Some(Token {
                token_type: TokenType::NewLine,
                span: Span::new(start, self.index),
            });

            if *byte == self.current {
                self.advance();
            } else {
                break;
            }
        }

        token
    }

    fn parse_symbol(&mut self) -> Token {
        let start = self.index;
        let mut value = Vec::new();
        while !is_word_break(self.current) && !self.eof {
            value.push(self.current);
            self.advance();
        }
        let end = self.index;
        let span = Span::new(start, end);

        let value = String::from_utf8(value).unwrap();

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

    fn parse_number(&mut self) -> Token {
        let start = self.index;
        let mut value = Vec::new();
        loop {
            if self.eof || self.current == b'.' && self.peek(1) == Some(b'.') {
                break;
            }
            if is_word_break(self.current) && self.current != b'.' {
                break;
            }
            if memchr(self.current, b"*<>%!").is_some() {
                break;
            }
            if (value.get(..2) == Some(b"0x") || value.get(..2) == Some(b"0b"))
                && memchr(self.current, b"+-").is_some()
            {
                break;
            }
            let last = value.last().copied();
            if last != Some(b'e') && last != Some(b'E') && memchr(self.current, b"+-").is_some() {
                break;
            }
            if self.current != b'_' {
                value.push(self.current);
            }
            self.advance();
        }
        let end = self.index;
        let value = String::from_utf8(value).unwrap();
        let string = self.src()[start..end].to_string();

        let bytes = value.as_bytes();
        if bytes.get(..2) == Some(b"0x") {
            match BigUint::parse_bytes(&bytes[2..], 16) {
                Some(int) => {
                    return Token {
                        token_type: TokenType::Int(int, string),
                        span: Span::new(start, end),
                    }
                }
                None => {
                    return {
                        Token {
                            token_type: TokenType::Symbol(string),
                            span: Span::new(start, end),
                        }
                    }
                }
            }
        }

        if bytes.get(..2) == Some(b"0b") {
            match BigUint::parse_bytes(&bytes[2..], 2) {
                Some(int) => {
                    return Token {
                        token_type: TokenType::Int(int, string),
                        span: Span::new(start, end),
                    }
                }
                None => {
                    return {
                        Token {
                            token_type: TokenType::Symbol(string),
                            span: Span::new(start, end),
                        }
                    }
                }
            }
        }

        if let Ok(int) = value.parse::<BigUint>() {
            Token {
                token_type: TokenType::Int(int, string),
                span: Span::new(start, end),
            }
        } else if let Ok(deciaml) = value.parse::<BigDecimal>() {
            Token {
                token_type: TokenType::Float(deciaml, string),
                span: Span::new(start, end),
            }
        } else {
            Token {
                token_type: TokenType::Symbol(string),
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
                b'\\' => self.advance_with(TokenType::Symbol(String::from("\\")), 1),
                b'#' => self.advance_with(TokenType::Symbol(String::from("#")), 1),
                b'$' => self.advance_with(TokenType::Dollar, 1),
                b'?' => self.advance_with(TokenType::QuestionMark, 1),
                b'.' => {
                    if self.peek(1) == Some(b'.') {
                        self.advance_with(TokenType::Range, 2)
                    } else {
                        self.advance_with(TokenType::Dot, 1)
                    }
                }
                b',' => self.advance_with(TokenType::Comma, 1),
                b'|' => {
                    if self.peek(1) == Some(b'|') {
                        self.advance_with(TokenType::Or, 2)
                    } else {
                        self.advance_with(TokenType::Pipe, 1)
                    }
                }
                b'&' => {
                    if self.peek(1) == Some(b'&') {
                        self.advance_with(TokenType::And, 2)
                    } else {
                        self.advance_with(TokenType::Exec, 1)
                    }
                }
                b'@' => self.advance_with(TokenType::At, 1),
                b'\'' => self.advance_with(TokenType::Quote, 1),
                b'"' => self.advance_with(TokenType::DoubleQuote, 1),
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
                    match self.peek(1) {
                        Some(b'=') => return Some(self.advance_with(TokenType::Eq, 2)),
                        Some(b'~') => return Some(self.advance_with(TokenType::Match, 2)),
                        _ => (),
                    }

                    self.advance_with(TokenType::Assignment, 1)
                }
                b'+' => {
                    if self.peek(1) == Some(b'=') {
                        self.advance_with(TokenType::AddAssign, 2)
                    } else {
                        self.advance_with(TokenType::Add, 1)
                    }
                }
                b'-' => {
                    if self.peek(1) == Some(b'=') {
                        self.advance_with(TokenType::SubAssign, 2)
                    } else {
                        self.advance_with(TokenType::Sub, 1)
                    }
                }
                b'/' => {
                    if self.peek(1) == Some(b'=') {
                        self.advance_with(TokenType::DivAssign, 2)
                    } else {
                        self.advance_with(TokenType::Div, 1)
                    }
                }
                b'%' => {
                    if self.peek(1) == Some(b'=') {
                        self.advance_with(TokenType::ModAssign, 2)
                    } else {
                        self.advance_with(TokenType::Mod, 1)
                    }
                }
                b'*' => {
                    if self.peek(1) == Some(b'*') {
                        if self.peek(2) == Some(b'=') {
                            self.advance_with(TokenType::ExpoAssign, 3)
                        } else {
                            self.advance_with(TokenType::Expo, 2)
                        }
                    } else if self.peek(1) == Some(b'=') {
                        self.advance_with(TokenType::MulAssign, 2)
                    } else {
                        self.advance_with(TokenType::Mul, 1)
                    }
                }

                b'<' => {
                    if self.peek(1) == Some(b'=') {
                        self.advance_with(TokenType::Le, 2)
                    } else {
                        self.advance_with(TokenType::Lt, 1)
                    }
                }
                b'>' => {
                    if self.peek(1) == Some(b'=') {
                        self.advance_with(TokenType::Ge, 2)
                    } else {
                        self.advance_with(TokenType::Gt, 1)
                    }
                }
                b'!' => {
                    match self.peek(1) {
                        Some(b'=') => return Some(self.advance_with(TokenType::Ne, 2)),
                        Some(b'~') => return Some(self.advance_with(TokenType::NotMatch, 2)),
                        _ => (),
                    }
                    self.advance_with(TokenType::Not, 1)
                }
                _ => self.parse_symbol(),
            };
            return Some(token);
        }
        None
    }
}
