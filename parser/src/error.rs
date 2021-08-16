use crate::lexer::token::Token;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum SyntaxError {
    UnexpectedToken(Token),
    ExpectedToken,
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Self::UnexpectedToken(ref token) => write!(f, "unexpected token: {:?}", token),
            &Self::ExpectedToken => write!(f, "expected token"),
        }
    }
}

impl Error for SyntaxError {}
