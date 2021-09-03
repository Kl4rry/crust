use std::{collections::HashMap, ops::Range, rc::Rc};

use thin_string::ThinString;
use thin_vec::ThinVec;

use crate::parser::runtime_error::RunTimeError;

pub enum ValueKind {
    Heap(Rc<Value>),
    Stack(Value),
}

impl AsRef<Value> for ValueKind {
    #[inline(always)]
    fn as_ref(&self) -> &Value {
        match self {
            ValueKind::Heap(value) => value.as_ref(),
            ValueKind::Stack(value) => value,
        }
    }
}

impl From<Value> for ValueKind {
    #[inline(always)]
    fn from(value: Value) -> ValueKind {
        ValueKind::Stack(value)
    }
}

impl From<Rc<Value>> for ValueKind {
    #[inline(always)]
    fn from(value: Rc<Value>) -> ValueKind {
        ValueKind::Heap(value)
    }
}

#[allow(dead_code)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(ThinString),
    List(ThinVec<ValueKind>),
    Map(Box<HashMap<ValueKind, ValueKind>>),
    Range(Box<Range<i64>>),
    ExitStatus(i64),
}

impl AsRef<Value> for Value {
    #[inline(always)]
    fn as_ref(&self) -> &Value {
        self
    }
}

impl Value {
    pub fn try_to_string(&self) -> Result<String, RunTimeError> {
        match self {
            Self::Int(number) => Ok(number.to_string()),
            Self::Float(number) => Ok(number.to_string()),
            Self::String(string) => Ok(string.to_string()),
            Self::List(list) => {
                let mut vec: Vec<String> = Vec::new();
                for value in list.into_iter() {
                    vec.push(value.as_ref().try_to_string()?);
                }
                Ok(vec.join(" "))
            }
            Self::Range(range) => {
                let mut vec: Vec<i64> = Vec::new();
                vec.extend(range.clone().into_iter());

                Ok(vec
                    .into_iter()
                    .map(|x| {
                        let mut string = x.to_string();
                        string.push(' ');
                        string
                    })
                    .collect())
            }
            Self::ExitStatus(number) => Ok(number.to_string()),
            Self::Bool(boolean) => Ok(boolean.to_string()),
            _ => Err(RunTimeError::ConversionError),
        }
    }

    pub fn try_to_int(&self) -> Result<i64, RunTimeError> {
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
    }

    pub fn try_to_bool(&self) -> Result<bool, RunTimeError> {
        match self {
            Self::Int(number) => Ok(*number != 0),
            Self::Float(number) => Ok(*number != 0.0),
            Self::String(string) => {
                let res = string.parse();
                match res {
                    Ok(boolean) => Ok(boolean),
                    Err(_) => Err(RunTimeError::ConversionError),
                }
            }
            Self::ExitStatus(number) => Ok(*number == 0),
            Self::Bool(boolean) => Ok(*boolean),
            _ => Err(RunTimeError::ConversionError),
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            Self::String(string) => string.parse::<f64>().is_ok(),
            Self::Float(_) => true,
            _ => false,
        }
    }
}
