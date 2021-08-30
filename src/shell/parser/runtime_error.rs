use std::{error::Error, fmt};

use glob::{PatternError, GlobError};

#[derive(Debug)]
pub enum RunTimeError {
    TypeError,
    MaxRecursionError,
    OutOfIndexError,
    GlobError(GlobError),
    PatternError(PatternError),
    VariableNotFound,
    ConversionError,
}

impl fmt::Display for RunTimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<PatternError> for RunTimeError {
    fn from(error: PatternError) -> Self {
        RunTimeError::PatternError(error)
    }
}

impl From<GlobError> for RunTimeError {
    fn from(error: GlobError) -> Self {
        RunTimeError::GlobError(error)
    }
}

impl Error for RunTimeError {}
