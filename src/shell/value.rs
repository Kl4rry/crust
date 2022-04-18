use std::{fmt, hash::Hash, ops::Range};

use bitflags::bitflags;
use indexmap::IndexMap;
use yansi::Paint;

use super::stream::ValueStream;
use crate::parser::{ast::expr::binop::BinOp, shell_error::ShellErrorKind, P};

mod format;
pub mod table;
use table::Table;

bitflags! {
    #[rustfmt::skip]
    pub struct Type: u16 {
        const NULL =        0b0000000001;
        const INT =         0b0000000010;
        const FLOAT =       0b0000000100;
        const BOOL =        0b0000001000;
        const STRING =      0b0000010000;
        const LIST =        0b0000100000;
        const MAP =         0b0001000000;
        const TABLE =       0b0010000000;
        const RANGE =       0b0100000000;
        const VALUESTREAM = 0b1000000000;
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut is_first = false;

        if self.intersects(Self::NULL) {
            write!(f, "'null'")?;
            is_first = true;
        }

        if self.intersects(Self::INT) {
            if is_first {
                write!(f, " or ")?;
            }
            is_first = true;
            write!(f, "'int'")?;
        }

        if self.intersects(Self::FLOAT) {
            if is_first {
                write!(f, " or ")?;
            }
            is_first = true;
            write!(f, "'float'")?;
        }

        if self.intersects(Self::STRING) {
            if is_first {
                write!(f, " or ")?;
            }
            is_first = true;
            write!(f, "'string'")?;
        }

        if self.intersects(Self::LIST) {
            if is_first {
                write!(f, " or ")?;
            }
            is_first = true;
            write!(f, "'list'")?;
        }

        if self.intersects(Self::MAP) {
            if is_first {
                write!(f, " or ")?;
            }
            is_first = true;
            write!(f, "'map'")?;
        }

        if self.intersects(Self::TABLE) {
            if is_first {
                write!(f, " or ")?;
            }
            is_first = true;
            write!(f, "'table'")?;
        }

        if self.intersects(Self::RANGE) {
            if is_first {
                write!(f, " or ")?;
            }
            is_first = true;
            write!(f, "'range'")?;
        }

        if self.intersects(Self::VALUESTREAM) {
            if is_first {
                write!(f, " or ")?;
            }
            write!(f, "'stream'")?;
        }

        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Int(i128),
    Float(f64),
    Bool(bool),
    String(String),
    List(Vec<Value>),
    Map(P<IndexMap<String, Value>>),
    Table(P<Table>),
    Range(P<Range<i128>>),
    ValueStream(P<ValueStream>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(number) => number.fmt(f),
            Self::Float(number) => number.fmt(f),
            Self::String(string) => string.fmt(f),
            Self::List(list) => {
                if list.is_empty() {
                    return Ok(());
                }

                format::format_columns(f, (0..list.len()).map(Paint::green).zip(list))
            }
            Self::Map(map) => {
                if map.is_empty() {
                    return Ok(());
                }

                format::format_columns(f, map.iter())
            }
            Self::Table(table) => table.fmt(f),
            Self::Range(range) => {
                for i in range.clone() {
                    i.fmt(f)?;
                }
                Ok(())
            }
            Self::Bool(boolean) => boolean.fmt(f),
            Self::ValueStream(output) => output.fmt(f),
            _ => Ok(()),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Int(number) => match other {
                Value::Float(rhs) => *number as f64 == *rhs,
                Value::Int(rhs) => number == rhs,
                Value::Bool(rhs) => *number == *rhs as i128,
                _ => false,
            },
            Value::Float(number) => match other {
                Value::Float(rhs) => *number as f64 == *rhs,
                Value::Int(rhs) => *number == *rhs as f64,
                Value::Bool(rhs) => *number == *rhs as u8 as f64,
                _ => false,
            },
            Value::Bool(boolean) => match other {
                Value::Float(rhs) => *boolean as u8 as f64 == *rhs,
                Value::Int(rhs) => *boolean as i128 == *rhs,
                Value::Bool(rhs) => boolean == rhs,
                Value::String(string) => string.is_empty() != *boolean,
                Value::List(list) => list.is_empty() != *boolean,
                Value::Map(map) => map.is_empty() != *boolean,
                Value::Table(table) => table.is_empty() != *boolean,
                Value::Range(range) => (range.start == 0 && range.end == 0) != *boolean,
                Value::Null => false,
                Value::ValueStream(stream) => stream.is_empty() != *boolean,
            },
            Value::String(string) => match other {
                Value::String(rhs) => string == rhs,
                Value::Bool(rhs) => (string.len() == 1) == *rhs,
                _ => false,
            },
            Value::List(list) => match other {
                Value::List(rhs) => list == rhs,
                Value::Bool(rhs) => list.is_empty() != *rhs,
                _ => false,
            },
            Value::Map(map) => match other {
                Value::Map(rhs) => map == rhs,
                Value::Bool(rhs) => map.is_empty() != *rhs,
                _ => false,
            },
            Value::Table(table) => match other {
                Value::Table(rhs) => table == rhs,
                Value::Bool(rhs) => table.is_empty() != *rhs,
                _ => false,
            },
            Value::Range(range) => match other {
                Value::Range(rhs) => **range == **rhs,
                Value::Bool(rhs) => (range.start == 0 && range.end == 0) != *rhs,
                _ => false,
            },
            Value::Null => matches!(other, Value::Null),
            Value::ValueStream(_) => false,
        }
    }
}

impl Value {
    // this function should only be used for displaying values
    // only the display trait should ever call it
    // it should never be used just to convert a value to a string
    pub fn to_compact_string(&self) -> String {
        match self {
            Value::Null => String::new(),
            Self::Int(number) => Paint::yellow(number).to_string(),
            Self::Float(number) => Paint::yellow(number).to_string(),
            Self::String(string) => string.to_string(),
            Self::List(list) => format!("[list with {} items]", list.len()),
            Self::Map(map) => format!("[map with {} entries]", map.len()),
            Self::Table(table) => format!("[table with {} rows]", table.len()),
            Self::Range(range) => format!("[range from {} to {}]]", range.start, range.end),
            Self::Bool(boolean) => Paint::yellow(boolean).to_string(),
            Self::ValueStream(_) => String::from("[value stream]"),
        }
    }

    // this function converts a value to a string if it can be done so losslessly
    pub fn try_into_string(self) -> Result<String, ShellErrorKind> {
        match self {
            Self::Int(number) => Ok(number.to_string()),
            Self::Float(number) => Ok(number.to_string()),
            Self::String(string) => Ok(string),
            _ => Err(ShellErrorKind::InvalidConversion {
                from: self.to_type(),
                to: Type::STRING,
            }),
        }
    }

    pub fn try_add(self, rhs: Value) -> Result<Value, ShellErrorKind> {
        match self {
            Value::Int(number) => match rhs {
                Value::List(rhs) => {
                    let mut list: Vec<Value> = vec![self];
                    list.extend(rhs.into_iter());
                    Ok(Value::List(list))
                }
                Value::Float(rhs) => Ok(Value::Float(number as f64 + rhs)),
                _ => match rhs.try_as_int() {
                    Some(rhs) => Ok(Value::Int(number.wrapping_add(rhs))),
                    None => Err(ShellErrorKind::InvalidBinaryOperand(
                        BinOp::Add,
                        self.to_type(),
                        rhs.to_type(),
                    )),
                },
            },
            Value::Float(number) => match rhs {
                Value::List(rhs) => {
                    let mut list: Vec<Value> = vec![self];
                    list.extend(rhs.into_iter());
                    Ok(Value::List(list))
                }
                _ => match rhs.try_as_float() {
                    Some(rhs) => Ok(Value::Float(number + rhs)),
                    None => Err(ShellErrorKind::InvalidBinaryOperand(
                        BinOp::Add,
                        self.to_type(),
                        rhs.to_type(),
                    )),
                },
            },
            Value::Bool(boolean) => match rhs {
                Value::List(rhs) => {
                    let mut list: Vec<Value> = vec![self];
                    list.extend(rhs.into_iter());
                    Ok(Value::List(list))
                }
                Value::Float(rhs) => Ok(Value::Float(boolean as u8 as f64 + rhs)),
                _ => match rhs.try_as_int() {
                    Some(rhs) => Ok(Value::Int((boolean as i128).wrapping_add(rhs))),
                    None => Err(ShellErrorKind::InvalidBinaryOperand(
                        BinOp::Add,
                        self.to_type(),
                        rhs.to_type(),
                    )),
                },
            },
            Value::String(_) => {
                if let Value::List(rhs) = rhs {
                    let mut list: Vec<Value> = vec![self];
                    list.extend(rhs.into_iter());
                    return Ok(Value::List(list));
                }

                let rhs = match rhs {
                    Value::String(rhs) => rhs,
                    _ => {
                        return Err(ShellErrorKind::InvalidBinaryOperand(
                            BinOp::Add,
                            self.to_type(),
                            rhs.to_type(),
                        ))
                    }
                };

                let mut new = self.unwrap_string();
                new.push_str(&rhs);
                Ok(Value::String(new))
            }
            Value::List(_) => {
                let mut list = self.unwrap_list();
                list.push(rhs);
                Ok(Value::List(list))
            }
            _ => Err(ShellErrorKind::InvalidBinaryOperand(
                BinOp::Add,
                self.to_type(),
                rhs.to_type(),
            )),
        }
    }

    pub fn try_sub(self, rhs: Value) -> Result<Value, ShellErrorKind> {
        match self {
            Value::Int(number) => match rhs {
                Value::Int(rhs) => Ok(Value::Int(number.wrapping_sub(rhs))),
                Value::Float(rhs) => Ok(Value::Float(number as f64 - rhs)),
                _ => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOp::Sub,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Float(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float(number as f64 - rhs)),
                None => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOp::Sub,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Bool(boolean) => match rhs {
                Value::Int(rhs) => Ok(Value::Int((boolean as i128).wrapping_sub(rhs))),
                Value::Float(rhs) => Ok(Value::Float(boolean as u8 as f64 - rhs)),
                _ => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOp::Sub,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            _ => Err(ShellErrorKind::InvalidBinaryOperand(
                BinOp::Sub,
                self.to_type(),
                rhs.to_type(),
            )),
        }
    }

    pub fn try_mul(self, rhs: Value) -> Result<Value, ShellErrorKind> {
        let self_type = self.to_type();
        match self {
            Value::Int(number) => match rhs {
                Value::Int(rhs) => Ok(Value::Int(number.wrapping_mul(rhs))),
                Value::Float(rhs) => Ok(Value::Float(number as f64 * rhs)),
                Value::String(string) => {
                    if string.is_empty() {
                        return Ok(Value::String(string));
                    }

                    let mut new = String::new();
                    for _ in 0..number {
                        new.push_str(&string);
                    }
                    Ok(Value::String(new))
                }
                Value::List(list) => {
                    if list.is_empty() {
                        return Ok(Value::List(list));
                    }

                    let mut new = Vec::new();
                    for _ in 0..number {
                        new.extend_from_slice(&list);
                    }
                    Ok(Value::List(new))
                }
                _ => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOp::Mul,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Float(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float(number as f64 * rhs)),
                None => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOp::Mul,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Bool(boolean) => match rhs {
                Value::Int(rhs) => Ok(Value::Int((boolean as i128).wrapping_mul(rhs))),
                Value::Float(rhs) => Ok(Value::Float(boolean as u8 as f64 * rhs)),
                Value::String(string) => {
                    let mut new = String::new();
                    for _ in 0..boolean as u8 {
                        new.push_str(&string);
                    }
                    Ok(Value::String(new))
                }
                _ => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOp::Mul,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::String(string) => {
                if string.is_empty() {
                    return Ok(Value::String(string));
                }

                let mul = match rhs.try_as_int() {
                    Some(rhs) => rhs,
                    None => {
                        return Err(ShellErrorKind::InvalidBinaryOperand(
                            BinOp::Add,
                            self_type,
                            rhs.to_type(),
                        ))
                    }
                };
                let mut new = String::new();
                for _ in 0..mul {
                    new.push_str(&string);
                }
                Ok(Value::String(new))
            }
            Value::List(list) => {
                if list.is_empty() {
                    return Ok(Value::List(list));
                }

                let mul = match rhs.try_as_int() {
                    Some(rhs) => rhs,
                    None => {
                        return Err(ShellErrorKind::InvalidBinaryOperand(
                            BinOp::Add,
                            self_type,
                            rhs.to_type(),
                        ))
                    }
                };

                if list.is_empty() {
                    return Ok(Value::List(Vec::new()));
                }

                let mut new = Vec::new();
                for _ in 0..mul {
                    new.extend_from_slice(&list);
                }
                Ok(Value::List(new))
            }
            _ => Err(ShellErrorKind::InvalidBinaryOperand(
                BinOp::Mul,
                self.to_type(),
                rhs.to_type(),
            )),
        }
    }

    pub fn try_div(self, rhs: Value) -> Result<Value, ShellErrorKind> {
        if rhs.is_zero() {
            return Err(ShellErrorKind::DivisionByZero);
        }

        match self {
            Value::Int(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float(number as f64 / rhs)),
                None => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOp::Div,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Float(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float(number as f64 / rhs)),
                None => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOp::Div,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Bool(boolean) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float(boolean as u8 as f64 / rhs)),
                None => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOp::Div,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            _ => Err(ShellErrorKind::InvalidBinaryOperand(
                BinOp::Div,
                self.to_type(),
                rhs.to_type(),
            )),
        }
    }

    pub fn try_expo(self, rhs: Value) -> Result<Value, ShellErrorKind> {
        match self {
            Value::Int(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float((number as f64).powf(rhs))),
                None => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOp::Expo,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Float(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float((number).powf(rhs))),
                None => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOp::Expo,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Bool(boolean) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float((boolean as u8 as f64).powf(rhs))),
                None => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOp::Expo,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            _ => Err(ShellErrorKind::InvalidBinaryOperand(
                BinOp::Expo,
                self.to_type(),
                rhs.to_type(),
            )),
        }
    }

    pub fn try_mod(self, rhs: Value) -> Result<Value, ShellErrorKind> {
        if rhs.is_zero() {
            return Err(ShellErrorKind::DivisionByZero);
        }

        match self {
            Value::Int(number) => match rhs {
                Value::Int(rhs) => Ok(Value::Int(number % rhs)),
                Value::Float(rhs) => Ok(Value::Float(number as f64 % rhs)),
                _ => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOp::Mod,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Float(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float(number as f64 % rhs)),
                None => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOp::Mod,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Bool(boolean) => match rhs {
                Value::Int(rhs) => Ok(Value::Int(boolean as i128 % rhs as i128)),
                Value::Float(rhs) => Ok(Value::Float(boolean as u8 as f64 % rhs)),
                _ => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOp::Mod,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            _ => Err(ShellErrorKind::InvalidBinaryOperand(
                BinOp::Mod,
                self.to_type(),
                rhs.to_type(),
            )),
        }
    }

    pub fn to_type(&self) -> Type {
        match self {
            Self::Int(_) => Type::INT,
            Self::Float(_) => Type::FLOAT,
            Self::Bool(_) => Type::BOOL,
            Self::String(_) => Type::STRING,
            Self::List(_) => Type::LIST,
            Self::Map(_) => Type::MAP,
            Self::Table(_) => Type::TABLE,
            Self::Range(_) => Type::RANGE,
            Self::Null => Type::NULL,
            Self::ValueStream(_) => Type::VALUESTREAM,
        }
    }

    pub fn try_as_int(&self) -> Option<i128> {
        match self {
            Self::Int(number) => Some(*number),
            Self::Bool(boolean) => Some(*boolean as i128),
            _ => None,
        }
    }

    // these functions are bad and return the wrong error types
    // this results in unhelpful errors
    pub fn try_as_float(&self) -> Option<f64> {
        match self {
            Self::Int(number) => Some(*number as f64),
            Self::Float(number) => Some(*number),
            Self::Bool(boolean) => Some(*boolean as u8 as f64),
            _ => None,
        }
    }

    pub fn truthy(&self) -> bool {
        match self {
            Self::Int(number) => *number != 0,
            Self::Float(number) => *number != 0.0,
            Self::String(string) => !string.is_empty(),
            Self::Bool(boolean) => *boolean,
            Self::List(list) => !list.is_empty(),
            Self::Map(map) => !map.is_empty(),
            Self::Table(table) => !table.is_empty(),
            Self::Range(range) => range.start != 0 && range.end != 0,
            Self::Null => false,
            Self::ValueStream(stream) => !stream.is_empty(),
        }
    }

    pub fn unwrap_string(self) -> String {
        match self {
            Self::String(s) => s,
            _ => panic!(
                "called `Value::unwrap_string()` on a `{}` value",
                self.to_type()
            ),
        }
    }

    pub fn unwrap_int(&self) -> i128 {
        match self {
            Self::Int(s) => *s,
            _ => panic!(
                "called `Value::unwrap_int()` on a `{}` value",
                self.to_type()
            ),
        }
    }

    pub fn unwrap_list(self) -> Vec<Value> {
        match self {
            Self::List(s) => s,
            _ => panic!(
                "called `Value::unwrap_list()` on a `{}` value",
                self.to_type()
            ),
        }
    }

    pub fn unwrap_map(self) -> IndexMap<String, Value> {
        match self {
            Self::Map(s) => *s,
            _ => panic!(
                "called `Value::unwrap_map()` on a `{}` value",
                self.to_type()
            ),
        }
    }

    pub fn unwrap_float(&self) -> f64 {
        match self {
            Self::Float(s) => *s,
            _ => panic!(
                "called `Value::unwrap_float()` on a `{}` value",
                self.to_type()
            ),
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Value::Bool(false) | Value::Int(0) => true,
            Value::Float(number) if *number == 0.0 => true,
            _ => false,
        }
    }
}
