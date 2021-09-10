use std::ops::Range;

use thin_string::ThinString;
use thin_vec::ThinVec;

use super::HeapValue;
use crate::parser::runtime_error::RunTimeError;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(ThinString),
    List(ThinVec<HeapValue>),
    Range(Box<Range<i64>>),
    ExitStatus(i64),
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
            Self::ExitStatus(number) => number.to_string(),
            Self::Bool(boolean) => boolean.to_string(),
        }
    }
}

impl Value {
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
