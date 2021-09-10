use std::ops::Deref;
use std::rc::Rc;

pub mod value;
pub use value::Value;

#[derive(Debug)]
pub enum ValueKind {
    Heap(HeapValue),
    Stack(Value),
}

impl Deref for ValueKind {
    type Target = Value;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Heap(value) => &*value.ptr,
            Self::Stack(value) => value,
        }
    }
}

impl From<Value> for ValueKind {
    #[inline(always)]
    fn from(value: Value) -> ValueKind {
        ValueKind::Stack(value)
    }
}

impl From<HeapValue> for ValueKind {
    #[inline(always)]
    fn from(value: HeapValue) -> ValueKind {
        ValueKind::Heap(value)
    }
}

impl From<Value> for HeapValue {
    #[inline(always)]
    fn from(value: Value) -> HeapValue {
        HeapValue { ptr: Rc::new(value) }
    }
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct HeapValue {
    ptr: Rc<Value>,
}

impl Deref for HeapValue {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        &*self.ptr
    }
}

impl From<ValueKind> for HeapValue {
    #[inline(always)]
    fn from(value: ValueKind) -> HeapValue {
        match value {
            ValueKind::Heap(value) => value,
            ValueKind::Stack(value) => value.into(),
        }
    }
}
