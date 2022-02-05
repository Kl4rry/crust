use std::{error::Error, fmt};

use miette::{Diagnostic, LabeledSpan, NamedSource, SourceCode};

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

#[derive(Debug)]
pub struct SyntaxError {
    pub error: SyntaxErrorKind,
    pub src: NamedSource,
    pub len: usize,
}

impl Error for SyntaxError {}

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

    fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        Some(Box::new("Syntax Error"))
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
        }
    }
}
