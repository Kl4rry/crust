use std::{
    error::Error,
    fmt, io,
    num::{ParseFloatError, ParseIntError},
    path::PathBuf,
};

use glob::{GlobError, PatternError};
use miette::{Diagnostic, LabeledSpan, NamedSource, SourceCode};
use subprocess::{CommunicateError, PopenError};

use super::ast::expr::{binop::BinOp, unop::UnOp};
use crate::shell::value::{Type, Value};

#[derive(Debug)]
pub struct ShellError {
    pub error: ShellErrorKind,
    pub src: NamedSource,
    pub len: usize,
}

impl ShellError {
    pub fn new(error: ShellErrorKind, src: String, name: String) -> Self {
        ShellError {
            error,
            len: src.len(),
            src: NamedSource::new(name, src),
        }
    }

    pub fn is_exit(&self) -> bool {
        matches!(self.error, ShellErrorKind::Exit)
    }
}

impl Error for ShellError {}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.error.fmt(f)
    }
}

#[derive(Debug)]
pub enum ShellErrorKind {
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
    CommandPermissionDenied(String),
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
    Io(Option<PathBuf>, io::Error),
    Glob(GlobError),
    Pattern(PatternError),
    ParseInt(ParseIntError),
    ParseFloat(ParseFloatError),
    Popen(PopenError),
    Communicate(CommunicateError),
}

impl fmt::Display for ShellErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CommandNotFound(name) => write!(f, "Command '{}' not found", name),
            Self::CommandPermissionDenied(name) => write!(f, "Cannot run '{}' permission denied", name),
            Self::NoMatch(pattern) => write!(f, "No match found for pattern: '{}'", pattern),
            Self::VariableNotFound(name) => write!(f, "Variable with name: '{}' not found", name),
            Self::IntegerOverFlow => write!(f, "Integer literal too large"),
            Self::Interrupt => write!(f, "^C"),
            Self::InvalidPipelineInput { expected, got } => {
                write!(f, "Pipeline expected '{}' recived '{}'", expected, got)
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
                write!(f, "Cannot iterate over type '{}'", value)
            }
            Self::InvalidConversion { from, to } => {
                write!(f, "Cannot convert '{}' to '{}'", from, to)
            }
            Self::MaxRecursion(limit) => write!(f, "max recursion limit of {} reached", limit),
            Self::IndexOutOfBounds { length, index } => write!(
                f,
                "index is out of bounds, length is {} but the index is {}",
                length, index
            ),
            Self::Io(path, error) => match path {
                Some(path) => write!(f, "{} {}", error, path.to_string_lossy()),
                None => write!(f, "{}", error),
            },
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
            Self::Exit => unreachable!("exit should never be printed as an error"),
        }
    }
}

impl Diagnostic for ShellError {
    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        None
    }

    fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        Some(Box::new("Shell Error"))
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Error)
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src as &dyn SourceCode)
    }
}

impl From<PatternError> for ShellErrorKind {
    fn from(error: PatternError) -> Self {
        ShellErrorKind::Pattern(error)
    }
}

impl From<GlobError> for ShellErrorKind {
    fn from(error: GlobError) -> Self {
        ShellErrorKind::Glob(error)
    }
}

impl From<ParseIntError> for ShellErrorKind {
    fn from(error: ParseIntError) -> Self {
        ShellErrorKind::ParseInt(error)
    }
}

impl From<ParseFloatError> for ShellErrorKind {
    fn from(error: ParseFloatError) -> Self {
        ShellErrorKind::ParseFloat(error)
    }
}

impl From<PopenError> for ShellErrorKind {
    fn from(error: PopenError) -> Self {
        ShellErrorKind::Popen(error)
    }
}

impl From<CommunicateError> for ShellErrorKind {
    fn from(error: CommunicateError) -> Self {
        ShellErrorKind::Communicate(error)
    }
}

impl Error for ShellErrorKind {}
