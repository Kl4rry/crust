use std::collections::{vec_deque, VecDeque};
use std::fmt;

use super::value::Value;

#[derive(Debug, Default, Clone)]
pub struct ValueStream {
    values: VecDeque<Value>,
}

impl ValueStream {
    pub fn new() -> Self {
        Self {
            values: VecDeque::new(),
        }
    }

    pub fn from_value(value: Value) -> Self {
        let mut values = VecDeque::with_capacity(1);
        if value != Value::Null {
            values.push_back(value);
        }
        Self { values }
    }

    pub fn push(&mut self, value: Value) {
        self.values.push_back(value);
    }

    pub fn iter(&self) -> vec_deque::Iter<'_, Value> {
        self.values.iter()
    }

    pub fn extend<T: IntoIterator<Item = Value>>(&mut self, iter: T) {
        self.values.extend(iter)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl fmt::Display for ValueStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for value in self.values.iter() {
            value.fmt(f)?;
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl IntoIterator for ValueStream {
    type Item = Value;
    type IntoIter = impl Iterator<Item = Value>;
    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter().filter(|value| *value != Value::Null)
    }
}

#[derive(Debug, Default, Clone)]
pub struct OutputStream {
    pub stream: ValueStream,
    pub status: i32,
}

impl fmt::Display for OutputStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.stream.fmt(f)
    }
}

impl OutputStream {
    pub fn new(stream: ValueStream, status: i32) -> Self {
        Self { stream, status }
    }

    pub fn push(&mut self, value: Value) {
        self.stream.push(value);
    }

    pub fn extend<T: IntoIterator<Item = Value>>(&mut self, iter: T) {
        self.stream.extend(iter)
    }
}

impl IntoIterator for OutputStream {
    type Item = Value;
    type IntoIter = impl Iterator<Item = Value>;
    fn into_iter(self) -> Self::IntoIter {
        self.stream.into_iter()
    }
}
