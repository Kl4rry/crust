use std::{fmt, ops::Range, rc::Rc};

use indexmap::IndexMap;
use yansi::Paint;

use crate::parser::{ast::expr::binop::BinOp, shell_error::ShellErrorKind};

mod format;
pub mod table;
use table::Table;
mod types;
pub use types::Type;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(Rc<String>),
    List(Rc<Vec<Value>>),
    Map(Rc<IndexMap<String, Value>>),
    Table(Rc<Table>),
    Range(Rc<Range<i64>>),
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

                format::format_columns(f, (0..list.len()).map(Paint::green).zip(&**list))
            }
            Self::Map(map) => {
                if map.is_empty() {
                    return Ok(());
                }

                format::format_columns(f, map.iter())
            }
            Self::Table(table) => table.fmt(f),
            Self::Range(range) => {
                for i in (**range).clone() {
                    i.fmt(f)?;
                }
                Ok(())
            }
            Self::Bool(boolean) => boolean.fmt(f),
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
                Value::Map(map) => map.is_empty() != *boolean,
                Value::Table(table) => table.is_empty() != *boolean,
                Value::Range(range) => (range.start == 0 && range.end == 0) != *boolean,
                Value::Null => false,
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
        }
    }

    // this function converts a value to a string if it can be done so losslessly
    pub fn try_into_string(self) -> Result<String, ShellErrorKind> {
        match self {
            Self::Int(number) => Ok(number.to_string()),
            Self::Float(number) => Ok(number.to_string()),
            Self::String(string) => Ok(string.to_string()),
            _ => Err(ShellErrorKind::InvalidConversion {
                from: self.to_type(),
                to: Type::STRING,
            }),
        }
    }

    pub fn try_add(self, rhs: Value) -> Result<Value, ShellErrorKind> {
        match self {
            Value::Int(number) => match rhs {
                Value::List(mut list) => {
                    Rc::make_mut(&mut list).push(self);
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
                Value::List(mut list) => {
                    Rc::make_mut(&mut list).push(self);
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
                Value::List(mut list) => {
                    Rc::make_mut(&mut list).push(self);
                    Ok(Value::List(list))
                }
                Value::Float(rhs) => Ok(Value::Float(boolean as u8 as f64 + rhs)),
                _ => match rhs.try_as_int() {
                    Some(rhs) => Ok(Value::Int((boolean as i64).wrapping_add(rhs))),
                    None => Err(ShellErrorKind::InvalidBinaryOperand(
                        BinOp::Add,
                        self.to_type(),
                        rhs.to_type(),
                    )),
                },
            },
            Value::String(_) => {
                if let Value::List(mut list) = rhs {
                    Rc::make_mut(&mut list).push(self);
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
                Rc::make_mut(&mut new).push_str(&rhs);
                Ok(Value::String(new))
            }
            Value::List(mut list) => {
                Rc::make_mut(&mut list).push(rhs);
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
                Value::Int(rhs) => Ok(Value::Int((boolean as i64).wrapping_sub(rhs))),
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
                    Ok(Value::String(Rc::new(new)))
                }
                Value::List(list) => {
                    if list.is_empty() {
                        return Ok(Value::List(list));
                    }

                    let mut new = Vec::new();
                    for _ in 0..number {
                        new.extend_from_slice(&list);
                    }
                    Ok(Value::List(Rc::new(new)))
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
                Value::Int(rhs) => Ok(Value::Int((boolean as i64).wrapping_mul(rhs))),
                Value::Float(rhs) => Ok(Value::Float(boolean as u8 as f64 * rhs)),
                Value::String(string) => {
                    let mut new = String::new();
                    for _ in 0..boolean as u8 {
                        new.push_str(&string);
                    }
                    Ok(Value::String(Rc::new(new)))
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
                Ok(Value::String(Rc::new(new)))
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
                    return Ok(Value::List(Rc::new(Vec::new())));
                }

                let mut new = Vec::new();
                for _ in 0..mul {
                    new.extend_from_slice(&list);
                }
                Ok(Value::List(Rc::new(new)))
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
                Value::Int(rhs) => Ok(Value::Int(boolean as i64 % rhs as i64)),
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
        }
    }

    pub fn try_as_int(&self) -> Option<i64> {
        match self {
            Self::Int(number) => Some(*number),
            Self::Bool(boolean) => Some(*boolean as i64),
            _ => None,
        }
    }

    pub fn try_as_index(&self, len: usize) -> Result<usize, ShellErrorKind> {
        let len = len as i128;
        let index = match self {
            Self::Int(number) => *number as i128,
            Self::Bool(boolean) => *boolean as i128,
            _ => {
                return Err(ShellErrorKind::InvalidConversion {
                    from: self.to_type(),
                    to: Type::INT,
                })
            }
        };

        if index < 0 {
            let new_index = len + index;
            if new_index >= 0 {
                Ok(new_index as usize)
            } else {
                Err(ShellErrorKind::IndexOutOfBounds { len, index })
            }
        } else if index > usize::MAX as i128 {
            Err(ShellErrorKind::IndexOutOfBounds { len, index })
        } else if index < len {
            Ok(index as usize)
        } else {
            Err(ShellErrorKind::IndexOutOfBounds { len, index })
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

    pub fn unwrap_map(self) -> Rc<IndexMap<String, Value>> {
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
