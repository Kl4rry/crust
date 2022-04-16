use std::{fmt, hash::Hash, ops::Range};

use bitflags::bitflags;
use unicode_width::UnicodeWidthStr;
use yansi::Paint;

use super::stream::ValueStream;
use crate::parser::{ast::expr::binop::BinOp, shell_error::ShellErrorKind, P};

bitflags! {
    #[rustfmt::skip]
    pub struct Type: u8 {
        const NULL =        0b00000001;
        const INT =         0b00000010;
        const FLOAT =       0b00000100;
        const BOOL =        0b00001000;
        const STRING =      0b00010000;
        const LIST =        0b00100000;
        const RANGE =       0b01000000;
        const VALUESTREAM = 0b10000000;
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
                let mut longest = 0;
                let mut values = Vec::new();
                for value in list.iter() {
                    values.push(value.to_compact_string());
                    longest = std::cmp::max(
                        longest,
                        console::strip_ansi_codes(unsafe { values.last().unwrap_unchecked() })
                            .width_cjk(),
                    );
                }

                let index_len = values.len().to_string().len();

                {
                    let mut top = String::new();
                    top.push('╭');
                    for _ in 0..index_len + 2 {
                        top.push('─');
                    }
                    top.push('┬');
                    for _ in 0..longest + 2 {
                        top.push('─');
                    }
                    top.push_str("╮\n");
                    write!(f, "{}", Paint::rgb(171, 178, 191, top))?;
                }

                let bar = Paint::rgb(171, 178, 191, "│");
                for (index, value) in values.into_iter().enumerate() {
                    let index_spacing = index_len - index.to_string().len();
                    let value_spacing = longest - console::strip_ansi_codes(&value).width_cjk();
                    writeln!(
                        f,
                        "{bar} {:index_spacing$}{} {bar} {:value_spacing$}{} {bar}",
                        "",
                        Paint::green(index),
                        "",
                        value
                    )?;
                }

                {
                    let mut bot = String::new();
                    bot.push('╰');
                    for _ in 0..index_len + 2 {
                        bot.push('─');
                    }
                    bot.push('┴');
                    for _ in 0..longest + 2 {
                        bot.push('─');
                    }
                    bot.push_str("╯\n");
                    write!(f, "{}", Paint::rgb(171, 178, 191, bot))?;
                }

                Ok(())
            }
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
            Value::Null => Paint::yellow("null").to_string(),
            Self::Int(number) => Paint::yellow(number).to_string(),
            Self::Float(number) => Paint::yellow(number).to_string(),
            Self::String(string) => string.to_string(),
            Self::List(list) => format!("[list with {} items]", list.len()),
            Self::Range(range) => format!("[range from {} to {}]]", range.start, range.end),
            Self::Bool(boolean) => Paint::yellow(boolean).to_string(),
            Self::ValueStream(_) => String::from("[value stream]"),
        }
    }

    // this function converts a value if it can be done so losslessly
    pub fn try_as_string(&self) -> Result<String, ShellErrorKind> {
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
