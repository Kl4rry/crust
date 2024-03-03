use std::{
    fmt,
    io::{stdout, IsTerminal, Write},
    mem,
    rc::Rc,
    slice,
};

use super::value::Value;
use crate::parser::shell_error::ShellErrorKind;

#[derive(Debug, Default, Clone)]
pub struct ValueStream {
    values: Vec<Value>,
}

impl ValueStream {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn from_values(values: Vec<Value>) -> Self {
        debug_assert!(values.iter().all(|v| *v != Value::Null));
        Self { values }
    }

    pub fn from_value(value: Value) -> Self {
        let mut values = Vec::with_capacity(1);
        if value != Value::Null {
            values.push(value);
        }
        Self { values }
    }

    pub fn push(&mut self, value: Value) {
        if value != Value::Null {
            self.values.push(value);
        }
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.values.pop()
    }

    pub fn iter(&self) -> slice::Iter<'_, Value> {
        self.values.iter()
    }

    pub fn extend<T: IntoIterator<Item = Value>>(&mut self, iter: T) {
        self.values
            .extend(iter.into_iter().filter(|value| *value != Value::Null))
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn unpack(mut self) -> Value {
        match self.values.len() {
            0 => Value::Null,
            1 => unsafe { self.values.pop().unwrap_unchecked() },
            _ => Value::List(Rc::new(self.values)),
        }
    }

    pub fn take(&mut self) -> ValueStream {
        let mut replacement = ValueStream::new();
        mem::swap(&mut replacement, self);
        replacement
    }
}

impl fmt::Display for ValueStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.values.is_empty() {
            for value in self.values.iter() {
                value.fmt(f)?;
            }
        }
        Ok(())
    }
}

impl IntoIterator for ValueStream {
    type Item = Value;
    type IntoIter = impl Iterator<Item = Value>;
    fn into_iter(self) -> Self::IntoIter {
        self.values
            .into_iter()
            .filter(|value| *value != Value::Null)
    }
}

#[derive(Debug, Clone)]
pub struct OutputStream {
    inner: InnerStream,
}

impl OutputStream {
    pub fn new_capture() -> Self {
        Self {
            inner: InnerStream::Capture(Vec::new()),
        }
    }

    pub fn new_output() -> Self {
        Self {
            inner: InnerStream::Output(false),
        }
    }

    #[inline]
    pub fn push(&mut self, value: Value) -> Result<(), ShellErrorKind> {
        self.extend([value])
    }

    #[inline]
    pub fn push_value_stream(&mut self, stream: ValueStream) -> Result<(), ShellErrorKind> {
        self.extend(stream)
    }

    #[inline]
    pub fn extend<T: IntoIterator<Item = Value>>(&mut self, iter: T) -> Result<(), ShellErrorKind> {
        match &mut self.inner {
            InnerStream::Capture(values) => {
                values.extend(iter.into_iter().filter(|v| !v.is_null()))
            }
            InnerStream::Output(outputs) => {
                *outputs = true;
                let mut stdout = stdout();
                if stdout.is_terminal() {
                    for value in iter {
                        writeln!(stdout, "{}", value).map_err(|e| ShellErrorKind::Io(None, e))?;
                    }
                    stdout.flush().map_err(|e| ShellErrorKind::Io(None, e))?;
                } else {
                    let mut buffer = Vec::new();
                    for value in iter {
                        value.try_expand_to_strings_no_span(&mut buffer)?;
                        for s in &buffer {
                            writeln!(stdout, "{}", s).map_err(|e| ShellErrorKind::Io(None, e))?;
                        }
                        buffer.clear();
                    }
                }
            }
        }
        Ok(())
    }

    pub fn end(&mut self) {
        match &mut self.inner {
            InnerStream::Capture(_) => panic!("cannot end capture stream"),
            InnerStream::Output(output) => {
                if stdout().is_terminal() {
                    if *output {
                        if let Ok((x, _)) = crossterm::cursor::position() {
                            if x != 0 {
                                println!();
                            }
                        }
                    }
                    *output = false;
                }
            }
        }
    }

    pub fn is_capture(&self) -> bool {
        matches!(self.inner, InnerStream::Capture(_))
    }

    pub fn is_output(&self) -> bool {
        matches!(self.inner, InnerStream::Output(_))
    }

    pub fn into_value_stream(self) -> ValueStream {
        match self.inner {
            InnerStream::Capture(values) => ValueStream::from_values(values),
            InnerStream::Output(_) => panic!("cannot convert output to value stream"),
        }
    }
}

impl fmt::Display for OutputStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let InnerStream::Capture(ref values) = self.inner {
            for value in values.iter() {
                value.fmt(f)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
enum InnerStream {
    Capture(Vec<Value>),
    Output(bool),
}
