use std::{fmt, sync::Arc};

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
    ContinueOutsideLoop(Span),
    BreakOutsideLoop(Span),
    ReturnOutsideFunction(Span),
    ComparisonChaining(Span, Span),
}

impl fmt::Display for SyntaxErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnexpectedToken(_) => write!(f, "Unexpected token"),
            Self::ExpectedToken => write!(f, "Expected token"),
            Self::Regex(e, _) => e.fmt(f),
            Self::InvalidIdentifier(_) => write!(f, "Invalid identifier"),
            Self::InvalidHexEscape(_) => write!(f, "Invalid hex esacpe"),
            Self::ContinueOutsideLoop(_) => write!(f, "`continue` outside of loop"),
            Self::BreakOutsideLoop(_) => write!(f, "`break` outside of loop"),
            Self::ReturnOutsideFunction(_) => write!(f, "`return` outside of function"),
            Self::ComparisonChaining(_, _) => write!(f, "Comparison operators cannot be chained"),
        }
    }
}

#[derive(Debug, Error)]
pub struct SyntaxError {
    pub error: SyntaxErrorKind,
    pub src: Arc<NamedSource<String>>,
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
            ContinueOutsideLoop(span) => {
                LabeledSpan::new_with_span(Some(String::from("Continue must be used inside a loop")), *span)
            }
            BreakOutsideLoop(span) => {
                LabeledSpan::new_with_span(Some(String::from("Break must be used inside a loop")), *span)
            }
            ReturnOutsideFunction(span) => {
                LabeledSpan::new_with_span(Some(String::from("Return must be used inside a function")), *span)
            }
            ComparisonChaining(span1, span2) => {
                return Some(P::new(vec![LabeledSpan::new_with_span(None, *span1), LabeledSpan::new_with_span(None, *span2)].into_iter()))
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
    pub fn new(error: SyntaxErrorKind, src: Arc<NamedSource<String>>) -> Self {
        SyntaxError {
            error,
            len: src.inner().len(),
            src,
        }
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.error.fmt(f)
    }
}
