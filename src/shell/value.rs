use std::{
    fmt::{self, Write},
    ops::Range,
    rc::Rc,
};

use crossterm::style::Stylize;
use indexmap::IndexMap;
use regex::Regex;
use yansi::Paint;

use crate::parser::{
    ast::expr::binop::BinOpKind, lexer::token::span::Span, shell_error::ShellErrorKind,
};

mod format;
pub mod table;
use table::Table;
mod types;
pub use types::Type;

mod de;
mod ser;

#[derive(Debug, Clone)]
pub struct SpannedValue {
    pub value: Value,
    pub span: Span,
}

impl SpannedValue {
    pub fn try_as_index(&self, len: usize) -> Result<usize, ShellErrorKind> {
        let len = len as i128;
        let value = &self.value;
        let span = self.span;
        let index = match value {
            Value::Int(number) => *number as i128,
            Value::Bool(boolean) => *boolean as i128,
            _ => {
                return Err(ShellErrorKind::InvalidConversion {
                    from: value.to_type(),
                    to: Type::INT,
                    span,
                })
            }
        };

        if index < 0 {
            let new_index = len + index;
            if new_index >= 0 {
                Ok(new_index as usize)
            } else {
                Err(ShellErrorKind::IndexOutOfBounds { len, index, span })
            }
        } else if index > usize::MAX as i128 {
            Err(ShellErrorKind::IndexOutOfBounds { len, index, span })
        } else if index < len {
            Ok(index as usize)
        } else {
            Err(ShellErrorKind::IndexOutOfBounds { len, index, span })
        }
    }

    // this function converts a value to a string if it can be done so losslessly
    pub fn try_into_string(self) -> Result<String, ShellErrorKind> {
        let (value, span) = self.into();
        match value {
            Value::Int(number) => Ok(number.to_string()),
            Value::Float(number) => Ok(number.to_string()),
            Value::String(string) => Ok(string.to_string()),
            Value::Bool(boolean) => Ok(boolean.to_string()),
            _ => Err(ShellErrorKind::InvalidConversion {
                from: value.to_type(),
                to: Type::STRING,
                span,
            }),
        }
    }

    pub fn try_expand_to_strings(self, output: &mut Vec<String>) -> Result<(), ShellErrorKind> {
        let (value, span) = self.into();
        match value {
            Value::Int(number) => output.push(number.to_string()),
            Value::Float(number) => output.push(number.to_string()),
            Value::String(string) => output.push(string.to_string()),
            Value::Bool(boolean) => output.push(boolean.to_string()),
            Value::List(list) => {
                let list = Rc::try_unwrap(list).unwrap_or_else(|list| (*list).clone());
                for value in list {
                    value.spanned(span).try_expand_to_strings(output)?;
                }
            }
            _ => {
                return Err(ShellErrorKind::InvalidConversionContains {
                    from: value.to_type(),
                    to: Type::STRING,
                    span,
                })
            }
        }
        Ok(())
    }

    pub fn try_add(self, rhs: SpannedValue, binop: Span) -> Result<SpannedValue, ShellErrorKind> {
        let (lhs, lhs_span) = self.into();
        let (rhs, rhs_span) = rhs.into();
        let span = lhs_span + rhs_span;
        match lhs {
            Value::Int(number) => match rhs {
                Value::List(mut list) => {
                    Rc::make_mut(&mut list).push(lhs);
                    Ok(Value::List(list).spanned(span))
                }
                Value::Float(rhs) => Ok(Value::Float(number as f64 + rhs).spanned(span)),
                _ => match rhs.try_as_int() {
                    Some(rhs) => Ok(Value::Int(number.wrapping_add(rhs)).spanned(span)),
                    None => Err(ShellErrorKind::InvalidBinaryOperand(
                        BinOpKind::Add.spanned(binop),
                        lhs.to_type(),
                        rhs.to_type(),
                        lhs_span,
                        rhs_span,
                    )),
                },
            },
            Value::Float(number) => match rhs {
                Value::List(mut list) => {
                    Rc::make_mut(&mut list).push(lhs);
                    Ok(Value::List(list).spanned(span))
                }
                _ => match rhs.try_as_float() {
                    Some(rhs) => Ok(Value::Float(number + rhs).spanned(span)),
                    None => Err(ShellErrorKind::InvalidBinaryOperand(
                        BinOpKind::Add.spanned(binop),
                        lhs.to_type(),
                        rhs.to_type(),
                        lhs_span,
                        rhs_span,
                    )),
                },
            },
            Value::Bool(boolean) => match rhs {
                Value::List(mut list) => {
                    Rc::make_mut(&mut list).push(lhs);
                    Ok(Value::List(list).spanned(span))
                }
                Value::Float(rhs) => Ok(Value::Float(boolean as u8 as f64 + rhs).spanned(span)),
                _ => match rhs.try_as_int() {
                    Some(rhs) => Ok(Value::Int((boolean as i64).wrapping_add(rhs)).spanned(span)),
                    None => Err(ShellErrorKind::InvalidBinaryOperand(
                        BinOpKind::Add.spanned(binop),
                        lhs.to_type(),
                        rhs.to_type(),
                        lhs_span,
                        rhs_span,
                    )),
                },
            },
            Value::String(_) => {
                if let Value::List(mut list) = rhs {
                    Rc::make_mut(&mut list).push(lhs);
                    return Ok(Value::List(list).spanned(span));
                }

                let rhs = match rhs {
                    Value::String(rhs) => rhs,
                    _ => {
                        return Err(ShellErrorKind::InvalidBinaryOperand(
                            BinOpKind::Add.spanned(binop),
                            lhs.to_type(),
                            rhs.to_type(),
                            lhs_span,
                            rhs_span,
                        ))
                    }
                };

                let mut new = lhs.unwrap_string();
                Rc::make_mut(&mut new).push_str(&rhs);
                Ok(Value::String(new).spanned(span))
            }
            Value::List(mut list) => {
                Rc::make_mut(&mut list).push(rhs);
                Ok(Value::List(list).spanned(span))
            }
            _ => Err(ShellErrorKind::InvalidBinaryOperand(
                BinOpKind::Add.spanned(binop),
                lhs.to_type(),
                rhs.to_type(),
                lhs_span,
                rhs_span,
            )),
        }
    }

    pub fn try_sub(self, rhs: SpannedValue, binop: Span) -> Result<SpannedValue, ShellErrorKind> {
        let (lhs, lhs_span) = self.into();
        let (rhs, rhs_span) = rhs.into();
        let span = lhs_span + rhs_span;
        match lhs {
            Value::Int(number) => match rhs {
                Value::Int(rhs) => Ok(Value::Int(number.wrapping_sub(rhs)).spanned(span)),
                Value::Float(rhs) => Ok(Value::Float(number as f64 - rhs).spanned(span)),
                _ => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOpKind::Sub.spanned(binop),
                    lhs.to_type(),
                    rhs.to_type(),
                    lhs_span,
                    rhs_span,
                )),
            },
            Value::Float(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float(number - rhs).spanned(span)),
                None => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOpKind::Sub.spanned(binop),
                    lhs.to_type(),
                    rhs.to_type(),
                    lhs_span,
                    rhs_span,
                )),
            },
            Value::Bool(boolean) => match rhs {
                Value::Int(rhs) => Ok(Value::Int((boolean as i64).wrapping_sub(rhs)).spanned(span)),
                Value::Float(rhs) => Ok(Value::Float(boolean as u8 as f64 - rhs).spanned(span)),
                _ => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOpKind::Sub.spanned(binop),
                    lhs.to_type(),
                    rhs.to_type(),
                    lhs_span,
                    rhs_span,
                )),
            },
            _ => Err(ShellErrorKind::InvalidBinaryOperand(
                BinOpKind::Sub.spanned(binop),
                lhs.to_type(),
                rhs.to_type(),
                lhs_span,
                rhs_span,
            )),
        }
    }

    pub fn try_mul(self, rhs: SpannedValue, binop: Span) -> Result<SpannedValue, ShellErrorKind> {
        let (lhs, lhs_span) = self.into();
        let (rhs, rhs_span) = rhs.into();
        let ty_lhs = lhs.to_type();
        let span = lhs_span + rhs_span;
        match lhs {
            Value::Int(number) => match rhs {
                Value::Int(rhs) => Ok(Value::Int(number.wrapping_mul(rhs)).spanned(span)),
                Value::Float(rhs) => Ok(Value::Float(number as f64 * rhs).spanned(span)),
                Value::String(string) => {
                    if string.is_empty() {
                        return Ok(Value::String(string).spanned(span));
                    }

                    let mut new = String::new();
                    for _ in 0..number {
                        new.push_str(&string);
                    }
                    Ok(Value::from(new).spanned(span))
                }
                Value::List(list) => {
                    if list.is_empty() {
                        return Ok(Value::List(list).spanned(span));
                    }

                    let mut new = Vec::new();
                    for _ in 0..number {
                        new.extend_from_slice(&list);
                    }
                    Ok(Value::from(new).spanned(span))
                }
                _ => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOpKind::Mul.spanned(binop),
                    lhs.to_type(),
                    rhs.to_type(),
                    lhs_span,
                    rhs_span,
                )),
            },
            Value::Float(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float(number * rhs).spanned(span)),
                None => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOpKind::Mul.spanned(binop),
                    lhs.to_type(),
                    rhs.to_type(),
                    lhs_span,
                    rhs_span,
                )),
            },
            Value::Bool(boolean) => match rhs {
                Value::Int(rhs) => Ok(Value::Int((boolean as i64).wrapping_mul(rhs)).spanned(span)),
                Value::Float(rhs) => Ok(Value::Float(boolean as u8 as f64 * rhs).spanned(span)),
                Value::String(string) => {
                    let mut new = String::new();
                    for _ in 0..boolean as u8 {
                        new.push_str(&string);
                    }
                    Ok(Value::from(new).spanned(span))
                }
                _ => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOpKind::Mul.spanned(binop),
                    lhs.to_type(),
                    rhs.to_type(),
                    lhs_span,
                    rhs_span,
                )),
            },
            Value::String(string) => {
                if string.is_empty() {
                    return Ok(Value::String(string).spanned(span));
                }

                let mul = match rhs.try_as_int() {
                    Some(rhs) => rhs,
                    None => {
                        return Err(ShellErrorKind::InvalidBinaryOperand(
                            BinOpKind::Mul.spanned(binop),
                            ty_lhs,
                            rhs.to_type(),
                            lhs_span,
                            rhs_span,
                        ))
                    }
                };
                let mut new = String::new();
                for _ in 0..mul {
                    new.push_str(&string);
                }
                Ok(Value::from(new).spanned(span))
            }
            Value::List(list) => {
                if list.is_empty() {
                    return Ok(Value::List(list).spanned(span));
                }

                let mul = match rhs.try_as_int() {
                    Some(rhs) => rhs,
                    None => {
                        return Err(ShellErrorKind::InvalidBinaryOperand(
                            BinOpKind::Mul.spanned(binop),
                            ty_lhs,
                            rhs.to_type(),
                            lhs_span,
                            rhs_span,
                        ))
                    }
                };

                if list.is_empty() {
                    return Ok(Value::from(Vec::<Value>::new()).spanned(span));
                }

                let mut new = Vec::new();
                for _ in 0..mul {
                    new.extend_from_slice(&list);
                }
                Ok(Value::from(new).spanned(span))
            }
            _ => Err(ShellErrorKind::InvalidBinaryOperand(
                BinOpKind::Mul.spanned(binop),
                lhs.to_type(),
                rhs.to_type(),
                lhs_span,
                rhs_span,
            )),
        }
    }

    pub fn try_div(self, rhs: SpannedValue, binop: Span) -> Result<SpannedValue, ShellErrorKind> {
        let (lhs, lhs_span) = self.into();
        let (rhs, rhs_span) = rhs.into();
        let span = lhs_span + rhs_span;

        if rhs.is_zero() {
            return Err(ShellErrorKind::DivisionByZero);
        }

        match lhs {
            Value::Int(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float(number as f64 / rhs).spanned(span)),
                None => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOpKind::Div.spanned(binop),
                    lhs.to_type(),
                    rhs.to_type(),
                    lhs_span,
                    rhs_span,
                )),
            },
            Value::Float(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float(number / rhs).spanned(span)),
                None => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOpKind::Div.spanned(binop),
                    lhs.to_type(),
                    rhs.to_type(),
                    lhs_span,
                    rhs_span,
                )),
            },
            Value::Bool(boolean) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float(boolean as u8 as f64 / rhs).spanned(span)),
                None => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOpKind::Div.spanned(binop),
                    lhs.to_type(),
                    rhs.to_type(),
                    lhs_span,
                    rhs_span,
                )),
            },
            _ => Err(ShellErrorKind::InvalidBinaryOperand(
                BinOpKind::Div.spanned(binop),
                lhs.to_type(),
                rhs.to_type(),
                lhs_span,
                rhs_span,
            )),
        }
    }

    pub fn try_expo(self, rhs: SpannedValue, binop: Span) -> Result<SpannedValue, ShellErrorKind> {
        let (lhs, lhs_span) = self.into();
        let (rhs, rhs_span) = rhs.into();
        let span = lhs_span + rhs_span;

        match lhs {
            Value::Int(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float((number as f64).powf(rhs)).spanned(span)),
                None => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOpKind::Expo.spanned(binop),
                    lhs.to_type(),
                    rhs.to_type(),
                    lhs_span,
                    rhs_span,
                )),
            },
            Value::Float(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float((number).powf(rhs)).spanned(span)),
                None => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOpKind::Expo.spanned(binop),
                    lhs.to_type(),
                    rhs.to_type(),
                    lhs_span,
                    rhs_span,
                )),
            },
            Value::Bool(boolean) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float((boolean as u8 as f64).powf(rhs)).spanned(span)),
                None => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOpKind::Expo.spanned(binop),
                    lhs.to_type(),
                    rhs.to_type(),
                    lhs_span,
                    rhs_span,
                )),
            },
            _ => Err(ShellErrorKind::InvalidBinaryOperand(
                BinOpKind::Expo.spanned(binop),
                lhs.to_type(),
                rhs.to_type(),
                lhs_span,
                rhs_span,
            )),
        }
    }

    pub fn try_mod(self, rhs: SpannedValue, binop: Span) -> Result<SpannedValue, ShellErrorKind> {
        let (lhs, lhs_span) = self.into();
        let (rhs, rhs_span) = rhs.into();
        let span = lhs_span + rhs_span;

        if rhs.is_zero() {
            return Err(ShellErrorKind::DivisionByZero);
        }

        match lhs {
            Value::Int(number) => match rhs {
                Value::Int(rhs) => Ok(Value::Int(number % rhs).spanned(span)),
                Value::Float(rhs) => Ok(Value::Float(number as f64 % rhs).spanned(span)),
                _ => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOpKind::Mod.spanned(binop),
                    lhs.to_type(),
                    rhs.to_type(),
                    lhs_span,
                    rhs_span,
                )),
            },
            Value::Float(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float(number % rhs).spanned(span)),
                None => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOpKind::Mod.spanned(binop),
                    lhs.to_type(),
                    rhs.to_type(),
                    lhs_span,
                    rhs_span,
                )),
            },
            Value::Bool(boolean) => match rhs {
                Value::Int(rhs) => Ok(Value::Int(boolean as i64 % rhs).spanned(span)),
                Value::Float(rhs) => Ok(Value::Float(boolean as u8 as f64 % rhs).spanned(span)),
                _ => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOpKind::Mod.spanned(binop),
                    lhs.to_type(),
                    rhs.to_type(),
                    lhs_span,
                    rhs_span,
                )),
            },
            _ => Err(ShellErrorKind::InvalidBinaryOperand(
                BinOpKind::Mod.spanned(binop),
                lhs.to_type(),
                rhs.to_type(),
                lhs_span,
                rhs_span,
            )),
        }
    }

    pub fn try_match(self, rhs: SpannedValue, binop: Span) -> Result<bool, ShellErrorKind> {
        let (lhs, lhs_span) = self.into();
        let (rhs, rhs_span) = rhs.into();

        match &lhs {
            Value::String(string) => match rhs {
                Value::String(sub) => Ok(string.contains(&*sub)),
                Value::Regex(regex) => Ok(regex.0.is_match(string)),
                _ => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOpKind::Match.spanned(binop),
                    lhs.to_type(),
                    rhs.to_type(),
                    lhs_span,
                    rhs_span,
                )),
            },
            Value::List(list) => Ok(list.contains(&rhs)),
            Value::Map(map) => match &rhs {
                Value::String(key) => Ok(map.contains_key(key.as_str())),
                _ => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOpKind::Match.spanned(binop),
                    lhs.to_type(),
                    rhs.to_type(),
                    lhs_span,
                    rhs_span,
                )),
            },
            Value::Table(table) => match &rhs {
                Value::String(key) => Ok(table.has_column(key)),
                _ => Err(ShellErrorKind::InvalidBinaryOperand(
                    BinOpKind::Match.spanned(binop),
                    lhs.to_type(),
                    rhs.to_type(),
                    lhs_span,
                    rhs_span,
                )),
            },
            _ => Err(ShellErrorKind::InvalidBinaryOperand(
                BinOpKind::Match.spanned(binop),
                lhs.to_type(),
                rhs.to_type(),
                lhs_span,
                rhs_span,
            )),
        }
    }
}

impl AsRef<Value> for SpannedValue {
    fn as_ref(&self) -> &Value {
        &self.value
    }
}

impl From<SpannedValue> for Value {
    fn from(value: SpannedValue) -> Self {
        value.value
    }
}

impl From<SpannedValue> for (Value, Span) {
    fn from(value: SpannedValue) -> Self {
        (value.value, value.span)
    }
}

impl Value {
    pub fn spanned(self, span: Span) -> SpannedValue {
        SpannedValue { value: self, span }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(Rc<String>),
    List(Rc<Vec<Value>>),
    Map(Rc<IndexMap<Rc<str>, Value>>),
    Table(Rc<Table>),
    Range(Rc<Range<i64>>),
    Regex(Rc<(Regex, String)>),
    Binary(Rc<Vec<u8>>),
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

                format::format_columns(f, (0..list.len()).zip(&**list))
            }
            Self::Map(map) => {
                if map.is_empty() {
                    return Ok(());
                }

                format::format_columns(f, map.iter())
            }
            Self::Table(table) => table.fmt(f),
            Self::Range(range) => {
                format::format_columns(f, (**range).clone().zip((**range).clone().map(Value::from)))
            }
            Self::Bool(boolean) => boolean.fmt(f),
            Self::Regex(regex) => Paint::blue(format!("/{}/", &regex.1)).fmt(f),
            Self::Binary(bytes) => {
                let bytes = bytes.as_slice();

                for line in 0..(bytes.len() / 16 + 1) {
                    if bytes.len() > 0xFFFF_FFFF {
                        write!(f, "{}", format!("{:012x}:   ", line * 16).grey())?;
                    } else if bytes.len() > 0xFFFF {
                        write!(f, "{}", format!("{:08x}:   ", line * 16).grey())?;
                    } else {
                        write!(f, "{}", format!("{:04x}:   ", line * 16).grey())?;
                    }

                    let slice = &bytes[line * 16..(line * 16 + 16).min(bytes.len())];
                    for (i, byte) in slice.iter().copied().enumerate() {
                        let s = format!("{byte:02x}");
                        if byte == 0 {
                            write!(f, "{} ", s.dark_grey())?;
                        } else if byte.is_ascii_graphic() {
                            write!(f, "{} ", s.cyan())?;
                        } else if byte.is_ascii_whitespace() {
                            write!(f, "{} ", s.green())?;
                        } else if byte.is_ascii() {
                            write!(f, "{} ", s.red())?;
                        } else {
                            write!(f, "{} ", s.yellow())?;
                        }

                        if (i + 1) % 4 == 0 {
                            f.write_char(' ')?;
                        }
                    }

                    for _ in 0..16 - slice.len() {
                        f.write_str("   ")?;
                    }

                    for _ in 0..(16 - slice.len() / 4 + 1) {
                        f.write_char(' ')?;
                    }

                    f.write_str("  ")?;
                    for byte in slice.iter().copied() {
                        if byte == 0 {
                            write!(f, "{}", "0".dark_grey())?;
                        } else if byte.is_ascii_graphic() {
                            write!(f, "{}", &format!("{}", byte as char).cyan())?;
                        } else if byte.is_ascii_whitespace() {
                            write!(f, "{}", " ".green())?;
                        } else if byte.is_ascii() {
                            write!(f, "{}", "•".red())?;
                        } else {
                            write!(f, "{}", "x".yellow())?;
                        }
                    }

                    f.write_char('\n')?;
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl AsRef<Value> for Value {
    fn as_ref(&self) -> &Value {
        self
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Int(number) => match other {
                Value::Float(rhs) => *number as f64 == *rhs,
                Value::Int(rhs) => number == rhs,
                Value::Bool(rhs) => *number == *rhs as i64,
                _ => false,
            },
            Value::Float(number) => match other {
                Value::Float(rhs) => *number == *rhs,
                Value::Int(rhs) => *number == *rhs as f64,
                Value::Bool(rhs) => *number == *rhs as u8 as f64,
                _ => false,
            },
            Value::Bool(boolean) => match other {
                Value::Float(rhs) => *boolean as u8 as f64 == *rhs,
                Value::Int(rhs) => *boolean as i64 == *rhs,
                Value::Bool(rhs) => boolean == rhs,
                Value::String(string) => string.is_empty() != *boolean,
                Value::List(list) => list.is_empty() != *boolean,
                Value::Map(map) => map.is_empty() != *boolean,
                Value::Table(table) => table.is_empty() != *boolean,
                Value::Range(range) => (range.start == 0 && range.end == 0) != *boolean,
                Value::Binary(_) => false,
                Value::Null => false,
                Value::Regex(_) => false,
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
            Value::Binary(data) => match other {
                Value::Binary(rhs) => data == rhs,
                _ => false,
            },
            Value::Null => matches!(other, Value::Null),
            Value::Regex(regex) => match other {
                Value::Regex(other) => *regex.1 == *other.1,
                _ => false,
            },
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
            Self::Int(number) => number.to_string(),
            Self::Float(number) => number.to_string(),
            Self::String(string) => string.to_string(),
            Self::List(list) => format!("[list with {} items]", list.len()),
            Self::Map(map) => format!("[map with {} entries]", map.len()),
            Self::Table(table) => format!("[table with {} rows]", table.len()),
            Self::Range(range) => format!("[range from {} to {}]", range.start, range.end),
            Self::Bool(boolean) => boolean.to_string(),
            Self::Regex(regex) => format!("/{}/", regex.1),
            Self::Binary(data) => format!(
                "[{} of binary data]",
                humansize::format_size(data.len(), humansize::BINARY.space_after_value(false))
            ),
        }
    }

    pub fn compact_string_color(&self) -> comfy_table::Color {
        use comfy_table::Color;
        match self {
            Self::Int(_) | Self::Float(_) | Self::Bool(_) => Color::Yellow,
            Self::Regex(_) => Color::Blue,
            _ => Color::Reset,
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
            Self::Regex(..) => Type::REGEX,
            Self::Binary(..) => Type::BINARY,
        }
    }

    pub fn try_as_int(&self) -> Option<i64> {
        match self {
            Self::Int(number) => Some(*number),
            Self::Bool(boolean) => Some(*boolean as i64),
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
            Self::Binary(data) => !data.is_empty(),
            Self::Null => false,
            Self::Regex(..) => false,
        }
    }

    pub fn unwrap_string(self) -> Rc<String> {
        match self {
            Self::String(s) => s,
            _ => panic!(
                "called `Value::unwrap_string()` on a `{}` value",
                self.to_type()
            ),
        }
    }

    pub fn unwrap_as_str(&self) -> &str {
        match self {
            Self::String(s) => s,
            _ => panic!(
                "called `Value::unwrap_as_str()` on a `{}` value",
                self.to_type()
            ),
        }
    }

    pub fn unwrap_int(&self) -> i64 {
        match self {
            Self::Int(s) => *s,
            _ => panic!(
                "called `Value::unwrap_int()` on a `{}` value",
                self.to_type()
            ),
        }
    }

    pub fn unwrap_list(self) -> Rc<Vec<Value>> {
        match self {
            Self::List(s) => s,
            _ => panic!(
                "called `Value::unwrap_list()` on a `{}` value",
                self.to_type()
            ),
        }
    }

    pub fn unwrap_map(self) -> Rc<IndexMap<Rc<str>, Value>> {
        match self {
            Self::Map(s) => s,
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

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(Rc::new(value.to_string()))
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(Rc::new(value))
    }
}

impl From<IndexMap<Rc<str>, Value>> for Value {
    fn from(value: IndexMap<Rc<str>, Value>) -> Self {
        Value::Map(Rc::new(value))
    }
}

impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        Value::List(Rc::new(value))
    }
}

impl From<Table> for Value {
    fn from(value: Table) -> Self {
        Value::Table(Rc::new(value))
    }
}

impl From<Range<i64>> for Value {
    fn from(value: Range<i64>) -> Self {
        Value::Range(Rc::new(value))
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Int(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl From<char> for Value {
    fn from(value: char) -> Self {
        Value::String(Rc::new(String::from(value)))
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Self {
        Value::Binary(Rc::new(value))
    }
}
