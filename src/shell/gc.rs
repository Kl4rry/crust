use std::{cell::RefCell, collections::HashSet, ops::Deref};

pub mod value;
pub use value::Value;

thread_local! {
    pub static GC: Gc = Gc::new();
}

pub struct Gc {
    pub values: RefCell<Vec<*mut Value>>,
    pub keepers: RefCell<HashSet<*mut Value>>,
}

impl Gc {
    pub fn new() -> Self {
        Self {
            values: RefCell::new(Vec::new()),
            keepers: RefCell::new(HashSet::new()),
        }
    }
}

impl Default for Gc {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum ValueKind {
    Heap(HeapValue),
    Stack(Value),
}

impl Deref for ValueKind {
    type Target = Value;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Heap(value) => unsafe { &*value.ptr },
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
        let ptr = Box::into_raw(Box::new(value));
        GC.with(|gc| {
            gc.values.borrow_mut().push(ptr);
        });
        HeapValue { ptr }
    }
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct HeapValue {
    ptr: *mut Value,
}

impl HeapValue {
    pub fn trace(&self) {
        GC.with(|gc| {
            let mut keepers = gc.keepers.borrow_mut();
            if !keepers.contains(&self.ptr) {
                keepers.insert(self.ptr);
                drop(keepers);
                unsafe {
                    match &*self.ptr {
                        Value::List(items) => {
                            for item in items {
                                item.trace();
                            }
                        }
                        Value::Map(map) => {
                            for (key, value) in map.iter() {
                                key.trace();
                                value.trace();
                            }
                        }
                        _ => (),
                    }
                }
            }
        });
    }
}

impl Deref for HeapValue {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
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
