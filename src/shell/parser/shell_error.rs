use std::{
    fmt, io,
    num::{ParseFloatError, ParseIntError},
    path::PathBuf,
    sync::Arc,
};

use glob::{GlobError, PatternError};
use miette::{Diagnostic, LabeledSpan, SourceCode};
use rayon::prelude::*;
use subprocess::{CommunicateError, PopenError};
use thiserror::Error;

use super::{
    ast::expr::{binop::BinOp, unop::UnOp},
    lexer::token::span::Span,
    source::Source,
};
use crate::{
    argparse::ParseError,
    shell::{
        levenshtein::levenshtein_stripped,
        value::{SpannedValue, Type},
    },
    P,
};

#[derive(Debug, Error)]
pub struct ShellError {
    pub error: ShellErrorKind,
    pub src: Arc<Source>,
}

impl ShellError {
    pub fn new(error: ShellErrorKind, src: Arc<Source>) -> Self {
        ShellError { error, src }
    }

    pub fn is_exit(&self) -> bool {
        matches!(self.error, ShellErrorKind::Exit)
    }
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.error.fmt(f)
    }
}

#[derive(Debug, Error)]
pub enum ShellErrorKind {
    // exit, break, continue, and return are not real errors and are only used to interrupt execution
    // this is a not so nice hack but it works
    Exit,
    Break,
    Return(Option<SpannedValue>),
    Continue,

    // Interrupt indicates that the user has pressed ctrl-c
    Interrupt,

    // real errors
    Basic(&'static str, String),
    DivisionByZero,
    NoMatch(String, Span),
    MaxRecursion(usize),
    IndexOutOfBounds {
        len: i128,
        index: i128,
        span: Span,
    },
    ColumnNotFound(String),
    InvalidConversion {
        from: Type,
        to: Type,
        span: Span,
    },
    NoColumns(Type),
    NotIndexable(Type, Span),
    VariableNotFound(String),
    InvalidBinaryOperand(BinOp, Type, Type, Span, Span),
    InvalidUnaryOperand(UnOp, Type, Span),
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
    AssertionFailed,
    UnknownFileType(String),
    ArgParse(#[from] ParseError),
    Io(Option<PathBuf>, io::Error),
    Glob(#[from] GlobError),
    Pattern(#[from] PatternError),
    ParseInt(#[from] ParseIntError),
    ParseFloat(#[from] ParseFloatError),
    Popen(#[from] PopenError),
    Communicate(#[from] CommunicateError),
    Open(#[from] opener::OpenError),
    Ureq(#[from] ureq::Error),
    Json(#[from] serde_json::Error),
    TomlDe(#[from] toml::de::Error),
    TomlSer(#[from] toml::ser::Error),
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
            NoMatch(pattern, ..) => write!(f, "No match found for pattern '{pattern}'"),
            VariableNotFound(name) => write!(f, "Variable with name '{name}' not found"),
            IntegerOverFlow => write!(f, "Integer literal too large"),
            Interrupt => write!(f, "^C"),
            InvalidPipelineInput { expected, recived } => {
                write!(f, "Pipeline expected {expected} recived {recived}")
            }
            AssertionFailed => write!(f, "Assertion failed"),
            InvalidEnvVar(t) => write!(f, "Cannot assign type {t} to an environment variable"),
            ReadOnlyVar(name) => write!(f, "Cannot write to read only variable '{name}'"),
            ToFewArguments {
                name,
                expected,
                recived,
            } => {
                write!(f, "{name} expected {expected} arguments, recived {recived}")
            }
            NoColumns(t) => write!(f, "{t} does not have columns"),
            NotIndexable(t, ..) => write!(f, "Cannot index into {t}"),
            InvalidBinaryOperand(binop, lhs, rhs, ..) => {
                write!(f, "'{binop}' not supported between {lhs} and {rhs}",)
            }
            InvalidUnaryOperand(unop, value, ..) => {
                write!(f, "'{unop}' not supported for {value}")
            }
            InvalidIterator(value) => {
                write!(f, "Cannot iterate over type {value}")
            }
            InvalidConversion { from, to, .. } => {
                write!(f, "Cannot convert {from} to {to}")
            }
            MaxRecursion(limit) => write!(f, "Max recursion limit of {limit} reached"),
            IndexOutOfBounds { len, index, .. } => write!(
                f,
                "Index is out of bounds, length is {len} but the index is {index}"
            ),
            ColumnNotFound(column) => write!(f, "Column '{column}' not found"),
            UnknownFileType(ext) => write!(f, "Unkown file type `.{ext}`"),
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
            Json(error) => error.fmt(f),
            TomlDe(error) => error.fmt(f),
            TomlSer(error) => error.fmt(f),
            Open(error) => error.fmt(f),
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
        match self.error {
            ShellErrorKind::NotIndexable(ty, span) => Some(P::new(
                [LabeledSpan::new_with_span(Some(ty.to_string()), span)].into_iter(),
            )),
            ShellErrorKind::IndexOutOfBounds { span, .. } => Some(P::new(
                [LabeledSpan::new_with_span(
                    Some(String::from("Index is out of bounds")),
                    span,
                )]
                .into_iter(),
            )),
            ShellErrorKind::InvalidConversion { span, from, .. } => Some(P::new(
                [LabeledSpan::new_with_span(Some(from.to_string()), span)].into_iter(),
            )),
            ShellErrorKind::InvalidBinaryOperand(binop, lhs, rhs, lhs_span, rhs_span) => {
                let lhs = LabeledSpan::new_with_span(Some(lhs.to_string()), lhs_span);
                let binop = LabeledSpan::new_with_span(
                    Some(String::from("Not supported for these types")),
                    binop.span,
                );
                let rhs = LabeledSpan::new_with_span(Some(rhs.to_string()), rhs_span);
                Some(P::new([lhs, binop, rhs].into_iter()))
            }
            _ => None,
        }
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
            AssertionFailed => P::new("Assertion Error"),
            IndexOutOfBounds { .. } | ColumnNotFound(..) => P::new("Indexing Error"),
            Glob(..) | Pattern(..) | NoMatch(..) => P::new("Glob Error"),
            InvalidConversion { .. } => P::new("Coercion Error"),
            Ureq(..) => P::new("Http Error"),
            Break | Return(..) | Continue => P::new("Syntax Error"),
            Interrupt => P::new("Interrupt"),
            MaxRecursion(..) => P::new("Recursion Error"),
            CommandNotFound(..) | CommandPermissionDenied(..) => P::new("Command Error"),
            FileNotFound(..) | FilePermissionDenied(..) => P::new("File Error"),
            UnknownFileType(..) | TomlDe(..) | Json(..) => P::new("Deserialization Error"),
            TomlSer(..) => P::new("Serialization Error"),
            _ => P::new("Shell Error"),
        })
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(miette::Severity::Error)
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        Some(&self.src as &dyn SourceCode)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self.error {
            ShellErrorKind::CommandNotFound(ref cmd) => {
                let exes = executable_finder::executables().ok()?;

                let mut options: Vec<_> = exes
                    .par_iter()
                    .filter_map(|exec| {
                        let distance = levenshtein_stripped(&exec.name, cmd);
                        if distance < 10 {
                            Some((exec, distance))
                        } else {
                            None
                        }
                    })
                    .collect();
                options.sort_by_key(|(_, d)| *d);
                let closest = options.first()?;
                Some(P::new(format!("Did you mean {}?", closest.0.name)))
            }
            _ => None,
        }
    }
}
