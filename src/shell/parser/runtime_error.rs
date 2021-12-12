use std::{
    error::Error,
    fmt, io,
    num::{ParseFloatError, ParseIntError},
};

use glob::{GlobError, PatternError};
use subprocess::{CommunicateError, PopenError};

use super::ast::expr::{binop::BinOp, unop::UnOp};
use crate::shell::value::{Type, Value};

#[derive(Debug)]
pub enum RunTimeError {
    // exit, break, continue, and return are not real errors and are only used to interrupt execution
    // this is a not so nice hack but it works
    Exit,
    Break,
    Return(Option<Value>),
    Continue,

    // real errors
    NoMatch(String),
    MaxRecursion(usize),
    IndexOutOfBounds { length: usize, index: usize },
    InvalidConversion { from: Type, to: Type },
    VariableNotFound(String),
    InvalidBinaryOperand(BinOp, Type, Type),
    InvalidUnaryOperand(UnOp, Type),
    InvalidIterator(Type),
    CommandNotFound(String),
    Io(io::Error),
    Glob(GlobError),
    Pattern(PatternError),
    ParseInt(ParseIntError),
    ParseFloat(ParseFloatError),
    Popen(PopenError),
    Communicate(CommunicateError),
}

impl fmt::Display for RunTimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CommandNotFound(name) => write!(f, "{}: command not found", name),
            Self::NoMatch(pattern) => write!(f, "no match found for pattern: '{}'", pattern),
            Self::VariableNotFound(name) => write!(f, "variable with name: '{}' not found", name),
            Self::InvalidBinaryOperand(binop, lhs, rhs) => {
                write!(
                    f,
                    "'{}' not supported between '{}' and '{}'",
                    binop, lhs, rhs
                )
            }
            Self::InvalidUnaryOperand(unop, value) => {
                write!(f, "'{}' not supported for '{}'", unop, value)
            }
            Self::InvalidIterator(value) => {
                write!(f, "cannot iterate over type '{}'", value)
            }
            Self::InvalidConversion { from, to } => {
                write!(f, "cannot convert '{}' to '{}'", from, to)
            }
            Self::MaxRecursion(limit) => write!(f, "max recursion limit of {} reached", limit),
            Self::IndexOutOfBounds { length, index } => write!(
                f,
                "index is out of bounds, length is {} but the index is {}",
                length, index
            ),
            Self::Io(error) => error.fmt(f),
            Self::Glob(error) => error.fmt(f),
            Self::Pattern(error) => error.fmt(f),
            Self::ParseInt(error) => error.fmt(f),
            Self::ParseFloat(error) => error.fmt(f),
            Self::Communicate(error) => error.fmt(f),
            Self::Popen(error) => error.fmt(f),
            // exit should always be handled and should therefore never be displayed
            Self::Exit => unreachable!(),
            Self::Break => write!(f, "break must be used in loop"),
            Self::Return(_) => write!(f, "return must be used in function"),
            Self::Continue => write!(f, "continue must be used in loop"),
        }
    }
}

impl From<PatternError> for RunTimeError {
    fn from(error: PatternError) -> Self {
        RunTimeError::Pattern(error)
    }
}

impl From<GlobError> for RunTimeError {
    fn from(error: GlobError) -> Self {
        RunTimeError::Glob(error)
    }
}

impl From<std::io::Error> for RunTimeError {
    fn from(error: std::io::Error) -> Self {
        RunTimeError::Io(error)
    }
}

impl From<ParseIntError> for RunTimeError {
    fn from(error: ParseIntError) -> Self {
        RunTimeError::ParseInt(error)
    }
}

impl From<ParseFloatError> for RunTimeError {
    fn from(error: ParseFloatError) -> Self {
        RunTimeError::ParseFloat(error)
    }
}

impl From<PopenError> for RunTimeError {
    fn from(error: PopenError) -> Self {
        RunTimeError::Popen(error)
    }
}

impl From<CommunicateError> for RunTimeError {
    fn from(error: CommunicateError) -> Self {
        RunTimeError::Communicate(error)
    }
}

impl Error for RunTimeError {}
