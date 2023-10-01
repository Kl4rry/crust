use std::{hash::Hash, mem, ptr, rc::Rc};

use super::Value;

#[repr(transparent)]
pub struct HashableValue(pub Value);

impl Hash for HashableValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let value = &self.0;
        mem::discriminant(value).hash(state);
        match value {
            Value::Int(int) => int.hash(state),
            Value::Bool(boolean) => boolean.hash(state),
            Value::String(string) => string.hash(state),
            Value::List(list) => {
                for value in list.iter() {
                    value.as_hashable().hash(state);
                }
            }
            Value::Map(map) => {
                for (k, v) in map.iter() {
                    k.hash(state);
                    v.as_hashable().hash(state);
                }
            }
            Value::Table(table) => {
                for row in table.rows() {
                    for value in row {
                        value.as_hashable().hash(state);
                    }
                }
            }
            Value::Range(range) => {
                for value in (**range).clone() {
                    value.hash(state);
                }
            }
            Value::Regex(regex) => {
                let (_, string) = &**regex;
                string.hash(state);
            }
            Value::Binary(binary) => binary.hash(state),
            _ => (),
        }
        state.finish();
    }
}

impl PartialEq for HashableValue {
    fn eq(&self, other: &Self) -> bool {
        let lhs = &self.0;
        let other = &other.0;

        match lhs {
            Value::Int(number) => match other {
                Value::Int(rhs) => number == rhs,
                _ => false,
            },
            Value::Float(number) => match other {
                Value::Float(rhs) => *number == *rhs,
                _ => false,
            },
            Value::Bool(boolean) => match other {
                Value::Bool(rhs) => boolean == rhs,
                _ => false,
            },
            Value::String(string) => match other {
                Value::String(rhs) => string == rhs,
                _ => false,
            },
            Value::List(list) => match other {
                Value::List(rhs) => list == rhs,
                _ => false,
            },
            Value::Map(map) => match other {
                Value::Map(rhs) => map == rhs,
                _ => false,
            },
            Value::Table(table) => match other {
                Value::Table(rhs) => table == rhs,
                _ => false,
            },
            Value::Range(range) => match other {
                Value::Range(rhs) => **range == **rhs,
                _ => false,
            },
            Value::Binary(data) => match other {
                Value::Binary(rhs) => data == rhs,
                _ => false,
            },
            Value::Null => matches!(other, Value::Null),
            Value::Regex(regex) => match other {
                Value::Regex(other) => *regex.1 == *other.1,
                _ => false,
            },
            Value::Closure(closure) => match other {
                Value::Closure(rhs) => ptr::eq(Rc::as_ptr(closure), Rc::as_ptr(rhs)),
                _ => false,
            },
        }
    }
}

impl Eq for HashableValue {}
