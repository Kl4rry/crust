use super::lexer::token::Token;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(Token),
    MissingToken,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Self::UnexpectedToken(ref token) => write!(f, "unexpected token: {:?}", token),
            &Self::MissingToken => write!(f, "missing token"),
        }
    }
}

impl Error for ParseError {}
