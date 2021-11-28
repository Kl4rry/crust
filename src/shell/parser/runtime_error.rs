use std::{error::Error, fmt, io};

use glob::{GlobError, PatternError};

use super::ast::expr::{binop::BinOp, unop::UnOp};
use crate::shell::values::{value::Type, ValueKind};

#[derive(Debug)]
pub enum RunTimeError {
    // exit, break, continue, and return are not real errors and are only used to interrupt execution
    // this is a not so nice hack but it works
    Exit,
    Break,
    Return(Option<ValueKind>),
    Continue,

    // real errors
    NoMatch(String),
    MaxRecursion(usize),
    IndexOutOfBounds { length: usize, index: usize },
    InvalidConversion { from: Type, to: Type },
    VariableNotFound(String),
    InvalidBinaryOperand(BinOp, Type, Type),
    InvalidUnaryOperand(UnOp, Type),
    CommandNotFound(String),
    IoError(io::Error),
    GlobError(GlobError),
    PatternError(PatternError),
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
            Self::InvalidConversion { from, to } => {
                write!(f, "cannot convert '{}' to '{}'", from, to)
            }
            Self::MaxRecursion(limit) => write!(f, "max recursion limit of {} reached", limit),
            Self::IndexOutOfBounds { length, index } => write!(
                f,
                "index is out of bounds, length is {} but the index is {}",
                length, index
            ),
            Self::IoError(error) => error.fmt(f),
            Self::GlobError(error) => error.fmt(f),
            Self::PatternError(error) => error.fmt(f),
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

impl Error for RunTimeError {}
