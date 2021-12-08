use std::{fmt, ops::Range};

use thin_string::ThinString;
use thin_vec::ThinVec;

use crate::parser::runtime_error::RunTimeError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    String,
    List,
    Range,
}

impl Type {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Int => "int",
            Self::Float => "float",
            Self::Bool => "bool",
            Self::String => "string",
            Self::List => "list",
            Self::Range => "range",
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
    Int(i64),
    Float(f64),
    Bool(bool),
    String(ThinString),
    List(ThinVec<Value>),
    Range(Box<Range<i64>>),
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
        }
    }
}

impl Value {
    pub fn to_type(&self) -> Type {
        match self {
            Self::Int(_) => Type::Int,
            Self::Float(_) => Type::Float,
            Self::Bool(_) => Type::Bool,
            Self::String(_) => Type::String,
            Self::List(_) => Type::List,
            Self::Range(_) => Type::Range,
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

    /*pub fn try_to_int(&self) -> Result<i64, RunTimeError> {
        match self {
            Self::Int(number) => Ok(*number),
            Self::Float(number) => Ok(*number as i64),
            Self::String(string) => {
                let res = string.parse();
                match res {
                    Ok(number) => Ok(number),
                    Err(_) => Err(RunTimeError::ConversionError),
                }
            }
            Self::ExitStatus(number) => Ok(*number as i64),
            Self::Bool(boolean) => Ok(*boolean as i64),
            _ => Err(RunTimeError::ConversionError),
        }
    }

    pub fn try_to_float(&self) -> Result<f64, RunTimeError> {
        match self {
            Self::Int(number) => Ok(*number as f64),
            Self::Float(number) => Ok(*number),
            Self::String(string) => {
                let res = string.parse();
                match res {
                    Ok(number) => Ok(number),
                    Err(_) => Err(RunTimeError::ConversionError),
                }
            }
            Self::ExitStatus(number) => Ok(*number as f64),
            Self::Bool(boolean) => Ok(*boolean as i64 as f64),
            _ => Err(RunTimeError::ConversionError),
        }
    }*/

    pub fn truthy(&self) -> bool {
        match self {
            Self::Int(number) => *number != 0,
            Self::Float(number) => *number != 0.0,
            Self::String(string) => !string.is_empty(),
            Self::Bool(boolean) => *boolean,
            Self::List(list) => !list.is_empty(),
            Self::Range(range) => range.start != 0 && range.end != 0,
        }
    }

    /*pub fn is_float_not_int(&self) -> bool {
        match self {
            Self::String(string) => string.parse::<f64>().is_ok(),
            Self::Float(_) => true,
            _ => false,
        }
    }*/
}
