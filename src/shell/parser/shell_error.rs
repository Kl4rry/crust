use std::{
    error::Error,
    fmt, io,
    num::{ParseFloatError, ParseIntError},
    path::PathBuf,
};

use glob::{GlobError, PatternError};
use subprocess::{CommunicateError, PopenError};

use super::ast::expr::{binop::BinOp, unop::UnOp};
use crate::shell::value::{Type, Value};

#[derive(Debug)]
pub enum ShellError {
    // exit, break, continue, and return are not real errors and are only used to interrupt execution
    // this is a not so nice hack but it works
    Exit,
    Break,
    Return(Option<Value>),
    Continue,

    // Interrupt indicates that the user has pressed ctrl-c
    Interrupt,

    // real errors
    NoMatch(String),
    MaxRecursion(usize),
    IndexOutOfBounds {
        length: usize,
        index: usize,
    },
    InvalidConversion {
        from: Type,
        to: Type,
    },
    VariableNotFound(String),
    InvalidBinaryOperand(BinOp, Type, Type),
    InvalidUnaryOperand(UnOp, Type),
    InvalidIterator(Type),
    CommandNotFound(String),
    ToFewArguments {
        name: String,
        expected: usize,
        recived: usize,
    },
    IntegerOverFlow,
    InvalidPipelineInput {
        expected: Type,
        got: Type,
    },
    Io(PathBuf, io::Error),
    Glob(GlobError),
    Pattern(PatternError),
    ParseInt(ParseIntError),
    ParseFloat(ParseFloatError),
    Popen(PopenError),
    Communicate(CommunicateError),
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CommandNotFound(name) => write!(f, "{}: command not found", name),
            Self::NoMatch(pattern) => write!(f, "no match found for pattern: '{}'", pattern),
            Self::VariableNotFound(name) => write!(f, "variable with name: '{}' not found", name),
            Self::IntegerOverFlow => write!(f, "integer literal too large"),
            Self::Interrupt => write!(f, "^C"),
            Self::InvalidPipelineInput { expected, got } => {
                write!(f, "pipeline expected '{}' recived '{}'", expected, got)
            }
            Self::ToFewArguments {
                name,
                expected,
                recived,
            } => {
                write!(
                    f,
                    "{} expected {} arguments, recived {}",
                    name, expected, recived
                )
            }
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
            Self::Io(path, error) => {
                write!(f, "crust: {} {}", error.to_string(), path.to_string_lossy())
            }
            Self::Glob(error) => error.fmt(f),
            Self::Pattern(error) => error.fmt(f),
            Self::ParseInt(error) => error.fmt(f),
            Self::ParseFloat(error) => error.fmt(f),
            Self::Communicate(error) => error.fmt(f),
            Self::Popen(error) => error.fmt(f),
            Self::Break => write!(f, "break must be used in loop"),
            Self::Return(_) => write!(f, "return must be used in function"),
            Self::Continue => write!(f, "continue must be used in loop"),
            // exit should always be handled and should therefore never be displayed
            Self::Exit => unreachable!(),
        }
    }
}

impl From<PatternError> for ShellError {
    fn from(error: PatternError) -> Self {
        ShellError::Pattern(error)
    }
}

impl From<GlobError> for ShellError {
    fn from(error: GlobError) -> Self {
        ShellError::Glob(error)
    }
}

impl From<ParseIntError> for ShellError {
    fn from(error: ParseIntError) -> Self {
        ShellError::ParseInt(error)
    }
}

impl From<ParseFloatError> for ShellError {
    fn from(error: ParseFloatError) -> Self {
        ShellError::ParseFloat(error)
    }
}

impl From<PopenError> for ShellError {
    fn from(error: PopenError) -> Self {
        ShellError::Popen(error)
    }
}

impl From<CommunicateError> for ShellError {
    fn from(error: CommunicateError) -> Self {
        ShellError::Communicate(error)
    }
}

impl Error for ShellError {}
