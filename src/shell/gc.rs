use std::{collections::HashMap, ops::Range};

use thin_string::ThinString;
use thin_vec::ThinVec;

use crate::parser::runtime_error::RunTimeError;

#[allow(dead_code)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(ThinString),
    List(ThinVec<Value>),
    Map(Box<HashMap<Value, Value>>),
    Range(Box<Range<i64>>),
    ExitStatus(i64),
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
                    vec.push(value.try_to_string()?);
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
}
