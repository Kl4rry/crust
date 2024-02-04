pub mod token;

use std::sync::Arc;

use bigdecimal::{num_bigint::BigUint, BigDecimal};
use memchr::memchr;
use token::{span::Span, Token, TokenType};

use super::source::Source;
use crate::str_ext::StrExt;

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

fn is_word_break(ch: char) -> bool {
    const DISALLOWED: &str = "\0#$\"\'(){}[]|;&,.:\\/=";
    DISALLOWED.contains(ch)
        || ch.is_whitespace()
        || LINE_ENDINGS
            .iter()
            .any(|le| unsafe { le.chars().next().unwrap_unchecked() } == ch)
}

pub struct Lexer {
    src: Arc<Source>,
    index: usize,
}

impl Lexer {
    pub fn new(src: Arc<Source>) -> Self {
        Self { index: 0, src }
    }

    pub fn named_source(&self) -> Arc<Source> {
        self.src.clone()
    }

    #[inline(always)]
    pub fn src(&self) -> &str {
        &self.src.code
    }

    #[inline(always)]
    fn peek(&self, offset: i64) -> Option<u8> {
        self.src()
            .as_bytes()
            .get((self.index as i64 + offset) as usize)
            .copied()
    }

    #[inline(always)]
    fn advance(&mut self) {
        if self.index < self.src().len() {
            self.index += 1;
        }
    }

    #[inline(always)]
    fn advance_n(&mut self, steps: usize) {
        if self.index < self.src().len() {
            self.index += steps;
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
        loop {
            if let Some(current) = self.src().get_char(self.index) {
                if current.is_whitespace()
                    && LINE_ENDINGS
                        .iter()
                        .all(|le| unsafe { le.chars().next().unwrap_unchecked() } != current)
                {
                    advanced = true;
                    self.advance_n(current.len_utf8());
                    continue;
                }
            }
            break;
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
            .find(|le| le.chars().next() == self.src().get_char(self.index))?;
        for byte in ending.as_bytes() {
            if Some(byte) == self.src().as_bytes().get(self.index) {
                self.advance();
                token = Some(Token {
                    token_type: TokenType::NewLine,
                    span: Span::new(start, self.index),
                });
            } else {
                break;
            }
        }

        token
    }

    fn parse_symbol(&mut self) -> Token {
        let start = self.index;
        let mut value = String::new();
        loop {
            if let Some(ch) = self.src().get_char(self.index) {
                if !is_word_break(ch) {
                    value.push(ch);
                    self.advance_n(ch.len_utf8());
                    continue;
                }
            }
            break;
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

    fn parse_number(&mut self) -> Token {
        let start = self.index;
        let mut value = String::new();
        loop {
            let Some(current) = self.src().get_char(self.index) else {
                break;
            };
            if !current.is_ascii() {
                break;
            }
            if current == '.' && self.peek(1) == Some(b'.') {
                break;
            }
            if is_word_break(self.src().get_char(self.index).unwrap()) && current != '.' {
                break;
            }
            if memchr(current as u8, b"*<>%!").is_some() {
                break;
            }
            if (value.get(..2) == Some("0x") || value.get(..2) == Some("0b"))
                && memchr(current as u8, b"+-").is_some()
            {
                break;
            }
            let last = value.as_bytes().last().copied();
            if last != Some(b'e') && last != Some(b'E') && memchr(current as u8, b"+-").is_some() {
                break;
            }
            if current != '_' {
                value.push(current);
            }
            self.advance();
        }
        let end = self.index;
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
        if let Some(current) = self.peek(0) {
            if let Some(token) = self.parse_whitespace() {
                return Some(token);
            }

            if let Some(token) = self.parse_newline() {
                return Some(token);
            }

            if current.is_ascii_digit() {
                return Some(self.parse_number());
            }

            let token = match current {
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
