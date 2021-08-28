use std::{error::Error, fmt, iter};

use colored::Colorize;

use crate::lexer::token::{span::Span, Token};

#[derive(Debug)]
pub enum SyntaxErrorKind {
    UnexpectedToken(Token),
    ExpectedToken,
}

impl fmt::Display for SyntaxErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedToken(ref token) => write!(f, "unexpected token: {:?}", token),
            Self::ExpectedToken => write!(f, "expected token"),
        }
    }
}

impl Error for SyntaxErrorKind {}

#[derive(Debug)]
pub struct SyntaxError<'a> {
    error: SyntaxErrorKind,
    src: &'a str,
}

impl<'a> SyntaxError<'a> {
    pub fn new(error: SyntaxErrorKind, src: &'a str) -> Self {
        SyntaxError { error, src }
    }
}

impl<'a> fmt::Display for SyntaxError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.error {
            SyntaxErrorKind::UnexpectedToken(ref token) => {
                let line_span = get_line(&self.src, &token.span);
                let line = &self.src[line_span.start()..line_span.end()];
                let line_number = get_line_number(&self.src, &token.span);

                let spacing = String::from_utf8(
                    iter::repeat(b' ')
                        .take(line_number.to_string().len() + 1)
                        .collect(),
                )
                .unwrap();

                let start = token.span.start() - line_span.start();
                let marker_spacing = get_string(start, b' ');
                let marker = get_string(token.span.len(), b'^').red();

                writeln!(f, "{}: Unexpected token", "Syntax error".red())?;
                writeln!(f, "{}{}", spacing, "|".blue())?;
                writeln!(
                    f,
                    "{} {} {}",
                    line_number.to_string().blue(),
                    "|".blue(),
                    line
                )?;
                write!(f, "{}{} ", spacing, "|".blue())?;
                writeln!(f, "{}{}", marker_spacing, marker)
            }
            SyntaxErrorKind::ExpectedToken => write!(f, "expected token"),
        }
    }
}

impl<'a> Error for SyntaxError<'a> {}

fn get_line(src: &str, span: &Span) -> Span {
    let mut start = span.start();
    loop {
        if src.as_bytes()[start] == b'\n' {
            start += 1;
            break;
        } else {
            start -= 1;
        }
    }

    let mut end = span.start();
    while end < src.as_bytes().len() {
        if src.as_bytes()[end] == b'\n' {
            break;
        } else {
            end += 1;
        }
    }

    Span::new(start, end)
}

fn get_line_number(src: &str, span: &Span) -> usize {
    let haystack = &src[0..span.start()];
    bytecount::count(haystack.as_bytes(), b'\n') + 1
}

fn get_string(len: usize, byte: u8) -> String {
    String::from_utf8(iter::repeat(byte).take(len).collect()).unwrap()
}
