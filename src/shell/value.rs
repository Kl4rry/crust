use std::{
    fmt::{self, Write},
    ops::Range,
};

use num_traits::ops::wrapping::{WrappingAdd, WrappingMul, WrappingSub};

use super::stream::OutputStream;
use crate::parser::{ast::expr::binop::BinOp, runtime_error::RunTimeError, P};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Null,
    Int,
    Float,
    Bool,
    String,
    List,
    Range,
    OutputStream,
}

impl Type {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Int => "int",
            Self::Float => "float",
            Self::Bool => "bool",
            Self::String => "string",
            Self::List => "list",
            Self::Range => "range",
            Self::OutputStream => "output stream",
        }
    }
}

impl AsRef<str> for Type {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    List(Vec<Value>),
    Range(P<Range<i64>>),
    OutputStream(P<OutputStream>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(number) => number.fmt(f),
            Self::Float(number) => number.fmt(f),
            Self::String(string) => string.fmt(f),
            Self::List(list) => {
                //f.write_char('[')?;
                for value in list.iter() {
                    value.fmt(f)?;
                    f.write_char(' ')?;
                }
                //f.write_char(']')?;
                Ok(())
            }
            Self::Range(range) => {
                for i in range.clone() {
                    i.fmt(f)?;
                }
                Ok(())
            }
            Self::Bool(boolean) => boolean.fmt(f),
            Self::Null => Ok(()),
            Self::OutputStream(output) => output.fmt(f),
        }
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
                Value::Float(rhs) => *number as f64 == *rhs,
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
                Value::Range(range) => (range.start == 0 && range.end == 0) != *boolean,
                Value::Null => false,
                Value::OutputStream(stream) => stream.status == 0,
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
            Value::Range(range) => match other {
                Value::Range(rhs) => **range == **rhs,
                Value::Bool(rhs) => (range.start == 0 && range.end == 0) != *rhs,
                _ => false,
            },
            Value::Null => matches!(other, Value::Null),
            Value::OutputStream(_) => false,
        }
    }
}

impl AsRef<Value> for Value {
    #[inline(always)]
    fn as_ref(&self) -> &Value {
        self
    }
}

impl Value {
    pub fn try_add(self, rhs: Value) -> Result<Value, RunTimeError> {
        match self.as_ref() {
            Value::Int(number) => match rhs.as_ref() {
                Value::List(rhs) => {
                    let mut list: Vec<Value> = vec![self];
                    list.extend(rhs.iter().cloned());
                    Ok(Value::List(list))
                }
                Value::Float(rhs) => Ok(Value::Float(*number as f64 + *rhs)),
                _ => match rhs.try_as_int() {
                    Some(rhs) => Ok(Value::Int(number.wrapping_add(&rhs))),
                    None => Err(RunTimeError::InvalidBinaryOperand(
                        BinOp::Add,
                        self.to_type(),
                        rhs.to_type(),
                    )),
                },
            },
            Value::Float(number) => match rhs.as_ref() {
                Value::List(rhs) => {
                    let mut list: Vec<Value> = vec![self];
                    list.extend(rhs.iter().cloned());
                    Ok(Value::List(list))
                }
                _ => match rhs.try_as_float() {
                    Some(rhs) => Ok(Value::Float(number + rhs)),
                    None => Err(RunTimeError::InvalidBinaryOperand(
                        BinOp::Add,
                        self.to_type(),
                        rhs.to_type(),
                    )),
                },
            },
            Value::Bool(boolean) => match rhs.as_ref() {
                Value::List(rhs) => {
                    let mut list: Vec<Value> = vec![self];
                    list.extend(rhs.iter().cloned());
                    Ok(Value::List(list))
                }
                Value::Float(rhs) => Ok(Value::Float(*boolean as u8 as f64 + *rhs)),
                _ => match rhs.try_as_int() {
                    Some(rhs) => Ok(Value::Int((*boolean as i64).wrapping_add(rhs))),
                    None => Err(RunTimeError::InvalidBinaryOperand(
                        BinOp::Add,
                        self.to_type(),
                        rhs.to_type(),
                    )),
                },
            },
            Value::String(string) => {
                if let Value::List(rhs) = rhs.as_ref() {
                    let mut list: Vec<Value> = vec![self.clone()];
                    list.extend(rhs.iter().cloned());
                    return Ok(Value::List(list));
                }

                let rhs = match rhs {
                    Value::String(rhs) => rhs,
                    _ => {
                        return Err(RunTimeError::InvalidBinaryOperand(
                            BinOp::Add,
                            self.to_type(),
                            rhs.to_type(),
                        ))
                    }
                };

                let mut new = string.clone();
                new.push_str(&rhs);
                Ok(Value::String(new))
            }
            Value::List(lhs) => {
                let mut list = lhs.clone();
                list.push(rhs);
                Ok(Value::List(list))
            }
            _ => Err(RunTimeError::InvalidBinaryOperand(
                BinOp::Add,
                self.to_type(),
                rhs.to_type(),
            )),
        }
    }

    pub fn try_sub(self, rhs: Value) -> Result<Value, RunTimeError> {
        match self.as_ref() {
            Value::Int(number) => match rhs.as_ref() {
                Value::Int(rhs) => Ok(Value::Int(number.wrapping_sub(rhs))),
                Value::Float(rhs) => Ok(Value::Float(*number as f64 - rhs)),
                _ => Err(RunTimeError::InvalidBinaryOperand(
                    BinOp::Sub,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Float(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float(*number as f64 - rhs)),
                None => Err(RunTimeError::InvalidBinaryOperand(
                    BinOp::Sub,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Bool(boolean) => match rhs.as_ref() {
                Value::Int(rhs) => Ok(Value::Int((*boolean as i64).wrapping_sub(*rhs))),
                Value::Float(rhs) => Ok(Value::Float(*boolean as u8 as f64 - rhs)),
                _ => Err(RunTimeError::InvalidBinaryOperand(
                    BinOp::Sub,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            _ => Err(RunTimeError::InvalidBinaryOperand(
                BinOp::Sub,
                self.to_type(),
                rhs.to_type(),
            )),
        }
    }

    pub fn try_mul(self, rhs: Value) -> Result<Value, RunTimeError> {
        match self.as_ref() {
            Value::Int(number) => match rhs.as_ref() {
                Value::Int(rhs) => Ok(Value::Int(number.wrapping_mul(rhs))),
                Value::Float(rhs) => Ok(Value::Float(*number as f64 * rhs)),
                Value::String(string) => {
                    if string.is_empty() {
                        return Ok(Value::String(String::new()));
                    }

                    let mut new = String::new();
                    for _ in 0..*number {
                        new.push_str(string);
                    }
                    Ok(Value::String(new))
                }
                Value::List(list) => {
                    let mul = match rhs.try_as_int() {
                        Some(rhs) => rhs,
                        None => {
                            return Err(RunTimeError::InvalidBinaryOperand(
                                BinOp::Add,
                                self.to_type(),
                                rhs.to_type(),
                            ))
                        }
                    };

                    if list.is_empty() {
                        return Ok(Value::List(Vec::new()));
                    }

                    let mut new = Vec::new();
                    for _ in 0..mul {
                        new.extend_from_slice(list);
                    }
                    Ok(Value::List(new))
                }
                _ => Err(RunTimeError::InvalidBinaryOperand(
                    BinOp::Mul,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Float(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float(*number as f64 * rhs)),
                None => Err(RunTimeError::InvalidBinaryOperand(
                    BinOp::Mul,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Bool(boolean) => match rhs.as_ref() {
                Value::Int(rhs) => Ok(Value::Int((*boolean as i64).wrapping_mul(*rhs))),
                Value::Float(rhs) => Ok(Value::Float(*boolean as u8 as f64 * rhs)),
                Value::String(string) => {
                    let mut new = String::new();
                    for _ in 0..*boolean as i64 {
                        new.push_str(string);
                    }
                    Ok(Value::String(new))
                }
                _ => Err(RunTimeError::InvalidBinaryOperand(
                    BinOp::Mul,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::String(string) => {
                let mul = match rhs.try_as_int() {
                    Some(rhs) => rhs,
                    None => {
                        return Err(RunTimeError::InvalidBinaryOperand(
                            BinOp::Add,
                            self.to_type(),
                            rhs.to_type(),
                        ))
                    }
                };
                let mut new = String::new();
                for _ in 0..mul {
                    new.push_str(string);
                }
                Ok(Value::String(new))
            }
            Value::List(list) => {
                let mul = match rhs.try_as_int() {
                    Some(rhs) => rhs,
                    None => {
                        return Err(RunTimeError::InvalidBinaryOperand(
                            BinOp::Add,
                            self.to_type(),
                            rhs.to_type(),
                        ))
                    }
                };

                if list.is_empty() {
                    return Ok(Value::List(Vec::new()));
                }

                let mut new = Vec::new();
                for _ in 0..mul {
                    new.extend_from_slice(list);
                }
                Ok(Value::List(new))
            }
            _ => Err(RunTimeError::InvalidBinaryOperand(
                BinOp::Mul,
                self.to_type(),
                rhs.to_type(),
            )),
        }
    }

    pub fn try_div(self, rhs: Value) -> Result<Value, RunTimeError> {
        match self.as_ref() {
            Value::Int(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float(*number as f64 / rhs)),
                None => Err(RunTimeError::InvalidBinaryOperand(
                    BinOp::Div,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Float(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float(*number as f64 / rhs)),
                None => Err(RunTimeError::InvalidBinaryOperand(
                    BinOp::Div,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Bool(boolean) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float(*boolean as u8 as f64 / rhs)),
                None => Err(RunTimeError::InvalidBinaryOperand(
                    BinOp::Div,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            _ => Err(RunTimeError::InvalidBinaryOperand(
                BinOp::Div,
                self.to_type(),
                rhs.to_type(),
            )),
        }
    }

    pub fn try_expo(self, rhs: Value) -> Result<Value, RunTimeError> {
        match self.as_ref() {
            Value::Int(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float((*number as f64).powf(rhs))),
                None => Err(RunTimeError::InvalidBinaryOperand(
                    BinOp::Expo,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Float(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float((*number).powf(rhs))),
                None => Err(RunTimeError::InvalidBinaryOperand(
                    BinOp::Expo,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Bool(boolean) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float((*boolean as u8 as f64).powf(rhs))),
                None => Err(RunTimeError::InvalidBinaryOperand(
                    BinOp::Expo,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            _ => Err(RunTimeError::InvalidBinaryOperand(
                BinOp::Expo,
                self.to_type(),
                rhs.to_type(),
            )),
        }
    }

    pub fn try_mod(self, rhs: Value) -> Result<Value, RunTimeError> {
        match self.as_ref() {
            Value::Int(number) => match rhs.as_ref() {
                Value::Int(rhs) => Ok(Value::Int(number % rhs)),
                Value::Float(rhs) => Ok(Value::Float(*number as f64 % rhs)),
                _ => Err(RunTimeError::InvalidBinaryOperand(
                    BinOp::Mod,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Float(number) => match rhs.try_as_float() {
                Some(rhs) => Ok(Value::Float(*number as f64 % rhs)),
                None => Err(RunTimeError::InvalidBinaryOperand(
                    BinOp::Mod,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Bool(boolean) => match rhs.as_ref() {
                Value::Int(rhs) => Ok(Value::Int(*boolean as i64 % rhs)),
                Value::Float(rhs) => Ok(Value::Float(*boolean as u8 as f64 % rhs)),
                _ => Err(RunTimeError::InvalidBinaryOperand(
                    BinOp::Mod,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            _ => Err(RunTimeError::InvalidBinaryOperand(
                BinOp::Mod,
                self.to_type(),
                rhs.to_type(),
            )),
        }
    }

    pub fn to_type(&self) -> Type {
        match self {
            Self::Int(_) => Type::Int,
            Self::Float(_) => Type::Float,
            Self::Bool(_) => Type::Bool,
            Self::String(_) => Type::String,
            Self::List(_) => Type::List,
            Self::Range(_) => Type::Range,
            Self::Null => Type::Null,
            Self::OutputStream(_) => Type::OutputStream,
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
            Self::Range(range) => range.start != 0 && range.end != 0,
            Self::Null => false,
            Self::OutputStream(stream) => stream.status == 0,
        }
    }
}
