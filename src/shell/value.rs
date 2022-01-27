use std::{fmt, ops::Range};

use thin_string::{ThinString, ToThinString};
use thin_vec::{thin_vec, ThinVec};

use super::stream::OutputStream;
use crate::parser::{ast::expr::binop::BinOp, runtime_error::RunTimeError};

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
    String(ThinString),
    List(ThinVec<Value>),
    Range(Box<Range<i64>>),
    OutputStream(Box<OutputStream>),
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
                Value::Bool(rhs) => *number == *rhs as i64 as f64,
                _ => false,
            },
            Value::Bool(boolean) => match other {
                Value::Float(rhs) => *boolean as i64 as f64 == *rhs,
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
            Value::Null => match other {
                Value::Null => true,
                _ => false,
            },
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

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Self::Int(number) => number.to_string(),
            Self::Float(number) => number.to_string(),
            Self::String(string) => string.to_string(),
            Self::List(list) => {
                let mut string = String::new();
                for value in list.into_iter() {
                    string.push_str(&value.as_ref().to_string());
                    string.push(' ');
                }
                string
            }
            Self::Range(range) => {
                let mut vec: Vec<i64> = Vec::new();
                vec.extend(range.clone().into_iter());

                vec.into_iter()
                    .map(|x| {
                        let mut string = x.to_string();
                        string.push(' ');
                        string
                    })
                    .collect()
            }
            Self::Bool(boolean) => boolean.to_string(),
            Self::Null => String::from(""),
            Self::OutputStream(output) => {
                let mut string = String::new();
                for val in &output.stream.values {
                    if !matches!(val, Value::Null) {
                        string.push_str(&val.to_string());
                    }
                }
                string
            }
        }
    }
}

impl Value {
    pub fn try_add(self, rhs: Value) -> Result<Value, RunTimeError> {
        match self.as_ref() {
            Value::Int(number) => match rhs.as_ref() {
                Value::List(rhs) => {
                    let mut list: ThinVec<Value> = thin_vec![self];
                    list.extend(rhs.iter().cloned());
                    Ok(Value::List(list))
                }
                Value::String(string) => {
                    let mut thin_string = number.to_thin_string();
                    thin_string.push_str(string);
                    Ok(Value::String(thin_string))
                }
                Value::Float(rhs) => Ok(Value::Float(*number as f64 + *rhs)),
                _ => Ok(Value::Int(number + rhs.try_as_int()?)),
            },
            Value::Float(number) => match rhs.as_ref() {
                Value::List(rhs) => {
                    let mut list: ThinVec<Value> = thin_vec![self];
                    list.extend(rhs.iter().cloned());
                    Ok(Value::List(list))
                }
                Value::String(string) => {
                    let mut thin_string = number.to_thin_string();
                    thin_string.push_str(string);
                    Ok(Value::String(thin_string))
                }
                _ => Ok(Value::Float(number + rhs.try_as_float()?)),
            },
            Value::Bool(boolean) => match rhs.as_ref() {
                Value::List(rhs) => {
                    let mut list: ThinVec<Value> = thin_vec![self];
                    list.extend(rhs.iter().cloned());
                    Ok(Value::List(list))
                }
                Value::Float(rhs) => Ok(Value::Float(*boolean as i64 as f64 + *rhs)),
                Value::String(string) => {
                    let mut thin_string = boolean.to_thin_string();
                    thin_string.push_str(string);
                    Ok(Value::String(thin_string))
                }
                _ => Ok(Value::Int(*boolean as i64 + self.try_as_int()?)),
            },
            Value::String(string) => {
                if let Value::List(rhs) = rhs.as_ref() {
                    let mut list: ThinVec<Value> = thin_vec![self.clone()];
                    list.extend(rhs.iter().cloned());
                    return Ok(Value::List(list));
                }

                let mut new = string.clone();
                let rhs = rhs.to_string();
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
                Value::Int(rhs) => Ok(Value::Int(number - rhs)),
                Value::Float(rhs) => Ok(Value::Float(*number as f64 - rhs)),
                _ => Err(RunTimeError::InvalidBinaryOperand(
                    BinOp::Sub,
                    self.to_type(),
                    rhs.to_type(),
                )),
            },
            Value::Float(number) => Ok(Value::Float(*number as f64 - rhs.try_as_float()?)),
            Value::Bool(boolean) => match rhs.as_ref() {
                Value::Int(rhs) => Ok(Value::Int(*boolean as i64 - rhs)),
                Value::Float(rhs) => Ok(Value::Float(*boolean as i64 as f64 - rhs)),
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
                Value::Int(rhs) => Ok(Value::Int(number * rhs)),
                Value::Float(rhs) => Ok(Value::Float(*number as f64 * rhs)),
                Value::String(string) => {
                    let mut new = ThinString::new();
                    for _ in 0..*number {
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
            Value::Float(number) => Ok(Value::Float(*number as f64 * rhs.try_as_float()?)),
            Value::Bool(boolean) => match rhs.as_ref() {
                Value::Int(rhs) => Ok(Value::Int(*boolean as i64 * rhs)),
                Value::Float(rhs) => Ok(Value::Float(*boolean as i64 as f64 * rhs)),
                Value::String(string) => {
                    let mut new = ThinString::new();
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
                let mut new = ThinString::new();
                let mul = rhs.try_as_int()?;
                for _ in 0..mul {
                    new.push_str(string);
                }
                Ok(Value::String(new))
            }
            Value::List(list) => {
                let mut new = ThinVec::new();
                let mul = rhs.try_as_int()?;
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
            Value::Int(number) => Ok(Value::Float(*number as f64 / rhs.try_as_float()?)),
            Value::Float(number) => Ok(Value::Float(*number as f64 / rhs.try_as_float()?)),
            Value::Bool(boolean) => Ok(Value::Float(*boolean as i64 as f64 / rhs.try_as_float()?)),
            _ => Err(RunTimeError::InvalidBinaryOperand(
                BinOp::Div,
                self.to_type(),
                rhs.to_type(),
            )),
        }
    }

    pub fn try_expo(self, rhs: Value) -> Result<Value, RunTimeError> {
        match self.as_ref() {
            Value::Int(number) => Ok(Value::Float((*number as f64).powf(rhs.try_as_float()?))),
            Value::Float(number) => Ok(Value::Float((*number as f64).powf(rhs.try_as_float()?))),
            Value::Bool(boolean) => Ok(Value::Float(
                (*boolean as i64 as f64).powf(rhs.try_as_float()?),
            )),
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
            Value::Float(number) => Ok(Value::Float(*number as f64 % rhs.try_as_float()?)),
            Value::Bool(boolean) => match rhs.as_ref() {
                Value::Int(rhs) => Ok(Value::Int(*boolean as i64 % rhs)),
                Value::Float(rhs) => Ok(Value::Float(*boolean as i64 as f64 % rhs)),
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

    pub fn try_as_int(&self) -> Result<i64, RunTimeError> {
        match self {
            Self::Int(number) => Ok(*number),
            Self::Bool(boolean) => Ok(*boolean as i64),
            _ => Err(RunTimeError::InvalidConversion {
                from: self.to_type(),
                to: Type::Int,
            }),
        }
    }

    // these functions are bad and return the wrong error types
    // this results in unhelpful errors
    pub fn try_as_float(&self) -> Result<f64, RunTimeError> {
        match self {
            Self::Int(number) => Ok(*number as f64),
            Self::Float(number) => Ok(*number),
            Self::Bool(boolean) => Ok(*boolean as i64 as f64),
            _ => Err(RunTimeError::InvalidConversion {
                from: self.to_type(),
                to: Type::Float,
            }),
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
