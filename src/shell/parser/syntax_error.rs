use std::fmt;

use miette::{Diagnostic, LabeledSpan, NamedSource, SourceCode};
use thiserror::Error;

use super::lexer::token::{span::Span, Token};
use crate::P;

#[derive(Debug, Error)]
pub enum SyntaxErrorKind {
    UnexpectedToken(Token),
    ExpectedToken,
    Regex(regex::Error, Span),
    InvalidIdentifier(Span),
    InvalidHexEscape(Span),
}

impl fmt::Display for SyntaxErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedToken(ref token) => write!(f, "unexpected token: {:?}", token),
            Self::ExpectedToken => write!(f, "expected token"),
            Self::Regex(e, _) => e.fmt(f),
            Self::InvalidIdentifier(_) => write!(f, "invalid identifier"),
            Self::InvalidHexEscape(_) => write!(f, "invalid hex esacpe"),
        }
    }
}

#[derive(Debug, Error)]
pub struct SyntaxError {
    pub error: SyntaxErrorKind,
    pub src: NamedSource,
    pub len: usize,
}

impl Diagnostic for SyntaxError {
    fn labels(&self) -> Option<P<dyn Iterator<Item = LabeledSpan> + '_>> {
        use SyntaxErrorKind::*;
        let label = match &self.error {
            UnexpectedToken(token) => {
                LabeledSpan::new_with_span(Some(String::from("Unexpected token")), token.span)
            }
            Regex(e, span) => {
                LabeledSpan::new_with_span(Some(e.to_string()), *span)
            }
            ExpectedToken => LabeledSpan::new_with_span(
                Some(String::from("Expected token after here")),
                Span::new(self.len - 1, self.len),
            ),
            InvalidIdentifier(span) => {
                LabeledSpan::new_with_span(Some(String::from("Identifiers can only contain numbers, letters and underscores and must not start with a number")), *span)
            }
            InvalidHexEscape(span) => {
                LabeledSpan::new_with_span(Some(String::from("Hex esacpe must only use 0 to F and be 127 or below")), *span)
            }
        };
        Some(P::new(vec![label].into_iter()))
    }

    fn code<'a>(&'a self) -> Option<P<dyn fmt::Display + 'a>> {
        Some(P::new("Syntax Error"))
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Error)
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

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self.error {
            SyntaxErrorKind::UnexpectedToken(_) => f.write_str("Unexpected token"),
            SyntaxErrorKind::ExpectedToken => f.write_str("Expected token"),
            SyntaxErrorKind::Regex(_, _) => write!(f, "Regex error"),
            SyntaxErrorKind::InvalidIdentifier(_) => write!(f, "Invalid identifier"),
            SyntaxErrorKind::InvalidHexEscape(_) => write!(f, "Invalid hex escape"),
        }
    }
}
