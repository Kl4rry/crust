use std::cell::UnsafeCell;

use slab::Slab;

pub mod value;
pub use value::Value;

thread_local! {
    pub static GC: UnsafeCell<Gc> = UnsafeCell::new(Gc::new());
}

pub fn drop_all() {
    let ptr = GC.with(|cell| cell.get());
    unsafe {
        (*ptr).values.clear();
    }
}

pub struct Gc {
    values: Slab<Value>,
}

impl Gc {
    pub fn new() -> Self {
        Self {
            values: Slab::new(),
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

impl AsRef<Value> for ValueKind {
    #[inline(always)]
    fn as_ref(&self) -> &Value {
        match self {
            ValueKind::Heap(id) => id.as_ref(),
            ValueKind::Stack(value) => value,
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
        let ptr = GC.with(|cell| cell.get());
        unsafe {
            HeapValue {
                id: (*ptr).values.insert(value),
            }
        }
    }
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct HeapValue {
    id: usize,
}

impl AsRef<Value> for HeapValue {
    #[inline(always)]
    fn as_ref(&self) -> &Value {
        let ptr = GC.with(|cell| cell.get());
        unsafe { (*ptr).values.get(self.id).unwrap() }
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

/*use std::cell::{Ref, RefCell};

thread_local! {
    pub static GC: RefCell<Gc> = RefCell::new(Gc::new());
}

pub enum ValueRef<'a> {
    Heap(Ref<'a, Gc>, &'a Value),
    Stack(&'a Value),
}

impl ValueKind {
    pub fn get_ref(&self) -> ValueRef {
        match self {
            Self::Heap(value) => {
                let gc = GC2.get_or(|| Gc::new());
                let id = value.id;
                gc.values.get(id);
                todo!();
            }
            Self::Stack(value) => ValueRef::Stack(value),
        }
    }
}*/
