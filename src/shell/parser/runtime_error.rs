use std::{error::Error, fmt};

use glob::{GlobError, PatternError};

#[derive(Debug)]
pub enum RunTimeError {
    Exit,
    TypeError,
    MaxRecursionError,
    OutOfIndexError,
    NoMatchError,
    VariableNotFound,
    ConversionError,
    IoError(std::io::Error),
    GlobError(GlobError),
    PatternError(PatternError),
    ClapError(clap::Error),
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

impl From<std::io::Error> for RunTimeError {
    fn from(error: std::io::Error) -> Self {
        RunTimeError::IoError(error)
    }
}

impl From<clap::Error> for RunTimeError {
    fn from(error: clap::Error) -> Self {
        RunTimeError::ClapError(error)
    }
}

impl Error for RunTimeError {}
