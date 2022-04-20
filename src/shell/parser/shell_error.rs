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
use crate::{
    argparse::ParseError,
    shell::value::{Type, Value},
    P,
};

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
    Basic(&'static str, String),
    DivisionByZero,
    NoMatch(String),
    MaxRecursion(usize),
    IndexOutOfBounds {
        len: i128,
        index: i128,
    },
    ColumnNotFound(String),
    InvalidConversion {
        from: Type,
        to: Type,
    },
    NoColumns(Type),
    NotIndexable(Type),
    VariableNotFound(String),
    InvalidBinaryOperand(BinOp, Type, Type),
    InvalidUnaryOperand(UnOp, Type),
    InvalidIterator(Type),
    InvalidEnvVar(Type),
    ReadOnlyVar(String),
    CommandNotFound(String),
    CommandPermissionDenied(String),
    FileNotFound(String),
    FilePermissionDenied(String),
    ToFewArguments {
        name: String,
        expected: usize,
        recived: usize,
    },
    IntegerOverFlow,
    InvalidPipelineInput {
        expected: Type,
        recived: Type,
    },
    ArgParse(ParseError),
    Io(Option<PathBuf>, io::Error),
    Glob(GlobError),
    Pattern(PatternError),
    ParseInt(ParseIntError),
    ParseFloat(ParseFloatError),
    Popen(PopenError),
    Communicate(CommunicateError),
    Ureq(ureq::Error),
}

impl fmt::Display for ShellErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ShellErrorKind::*;
        match self {
            Basic(_, e) => write!(f, "{e}"),
            DivisionByZero => write!(f, "Division by zero."),
            ArgParse(e) => write!(f, "{e}"),
            FileNotFound(path) => write!(f, "Cannot open '{path}' file not found"),
            FilePermissionDenied(path) => write!(f, "Cannot open '{path}' permission denied"),
            CommandNotFound(name) => write!(f, "Command '{name}' not found"),
            CommandPermissionDenied(name) => {
                write!(f, "Cannot run '{name}' permission denied")
            }
            NoMatch(pattern) => write!(f, "No match found for pattern '{pattern}'"),
            VariableNotFound(name) => write!(f, "Variable with name '{name}' not found"),
            IntegerOverFlow => write!(f, "Integer literal too large"),
            Interrupt => write!(f, "^C"),
            InvalidPipelineInput { expected, recived } => {
                write!(f, "Pipeline expected {expected} recived {recived}")
            }
            InvalidEnvVar(t) => write!(f, "Cannot assign type {t} to environment variable"),
            ReadOnlyVar(name) => write!(f, "Cannot write to read only variable '{name}'"),
            ToFewArguments {
                name,
                expected,
                recived,
            } => {
                write!(f, "{name} expected {expected} arguments, recived {recived}")
            }
            NoColumns(t) => write!(f, "{t} does not have columns"),
            NotIndexable(t) => write!(f, "Cannot index into {t}"),
            InvalidBinaryOperand(binop, lhs, rhs) => {
                write!(f, "'{binop}' not supported between {lhs} and {rhs}",)
            }
            InvalidUnaryOperand(unop, value) => {
                write!(f, "'{unop}' not supported for {value}")
            }
            InvalidIterator(value) => {
                write!(f, "Cannot iterate over type {value}")
            }
            InvalidConversion { from, to } => {
                write!(f, "Cannot convert {from} to {to}")
            }
            MaxRecursion(limit) => write!(f, "Max recursion limit of {limit} reached"),
            IndexOutOfBounds { len, index } => write!(
                f,
                "Index is out of bounds, length is {len} but the index is {index}"
            ),
            ColumnNotFound(column) => write!(f, "Column '{column}' not found"),
            Io(path, error) => match path {
                Some(path) => write!(f, "{} {}", error, path.to_string_lossy()),
                None => write!(f, "{}", error),
            },
            Glob(error) => error.fmt(f),
            Pattern(error) => error.fmt(f),
            ParseInt(error) => error.fmt(f),
            ParseFloat(error) => error.fmt(f),
            Communicate(error) => error.fmt(f),
            Popen(error) => error.fmt(f),
            Ureq(error) => error.fmt(f),
            Break => write!(f, "break must be used in loop"),
            Return(_) => write!(f, "return must be used in function"),
            Continue => write!(f, "continue must be used in loop"),
            // exit should always be handled and should therefore never be displayed
            Exit => unreachable!("exit should never be printed as an error"),
        }
    }
}

impl Diagnostic for ShellError {
    fn labels(&self) -> Option<P<dyn Iterator<Item = LabeledSpan> + '_>> {
        None
    }

    fn code<'a>(&'a self) -> Option<P<dyn fmt::Display + 'a>> {
        use ShellErrorKind::*;
        Some(match self.error {
            Basic(n, _) => P::new(n),
            DivisionByZero => P::new("Division by Zero Error"),
            InvalidBinaryOperand(..)
            | InvalidUnaryOperand(..)
            | InvalidIterator(..)
            | InvalidEnvVar(..)
            | NoColumns(..)
            | NotIndexable(..)
            | InvalidPipelineInput { .. } => P::new("Type Error"),
            IndexOutOfBounds { .. } | ColumnNotFound(..) => P::new("Indexing Error"),
            Glob(..) | Pattern(..) | NoMatch(..) => P::new("Glob Error"),
            InvalidConversion { .. } => P::new("Coercion Error"),
            Ureq(..) => P::new("Http Error"),
            Break | Return(..) | Continue => P::new("Syntax Error"),
            Interrupt => P::new("Interrupt"),
            MaxRecursion(..) => P::new("Recursion Error"),
            CommandNotFound(..) | CommandPermissionDenied(..) => P::new("Command Error"),
            FileNotFound(..) | FilePermissionDenied(..) => P::new("File Error"),
            _ => P::new("Shell Error"),
        })
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Error)
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src as &dyn SourceCode)
    }
}

impl From<ureq::Error> for ShellErrorKind {
    fn from(error: ureq::Error) -> Self {
        ShellErrorKind::Ureq(error)
    }
}

impl From<ParseError> for ShellErrorKind {
    fn from(error: ParseError) -> Self {
        ShellErrorKind::ArgParse(error)
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
