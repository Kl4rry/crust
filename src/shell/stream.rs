use std::collections::VecDeque;

use super::value::Value;

#[derive(Debug, Default, Clone)]
pub struct ValueStream {
    pub values: VecDeque<Value>,
}

impl ValueStream {
    pub fn new() -> Self {
        Self {
            values: VecDeque::new(),
        }
    }

    pub fn from_value(value: Value) -> Self {
        let mut values = VecDeque::with_capacity(1);
        values.push_back(value);
        Self { values }
    }
}

#[derive(Debug, Default, Clone)]
pub struct OutputStream {
    pub stream: ValueStream,
    pub status: i32,
}

impl OutputStream {
    pub fn new(stream: ValueStream, status: i32) -> Self {
        Self { stream, status }
    }
}
