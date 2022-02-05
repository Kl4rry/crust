use std::{error::Error, fmt};

use miette::{Diagnostic, LabeledSpan, NamedSource, SourceCode};
use thiserror::Error;

use super::lexer::token::{span::Span, Token};

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

#[derive(Error, Debug)]
#[error("Syntax Error")]
pub struct SyntaxError {
    pub error: SyntaxErrorKind,
    pub src: NamedSource,
    pub len: usize,
}

impl Diagnostic for SyntaxError {
    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        let label = match self.error {
            SyntaxErrorKind::UnexpectedToken(ref token) => {
                LabeledSpan::new_with_span(Some(String::from("Unexpected token")), token.span)
            }
            SyntaxErrorKind::ExpectedToken => LabeledSpan::new_with_span(
                Some(String::from("Expected token after here")),
                Span::new(self.len - 1, self.len),
            ),
        };
        Some(Box::new(vec![label].into_iter()))
    }

    fn severity(&self) -> Option<miette::Severity> {
        None
    }

    fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        None
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src as &dyn SourceCode)
    }
}

impl SyntaxError {
    pub fn new(error: SyntaxErrorKind, src: String, name: String) -> Self {
        SyntaxError {
            error,
            len: src.len(),
            src: NamedSource::new(name, src),
        }
    }
}
