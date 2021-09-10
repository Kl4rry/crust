use std::collections::HashMap;
use crate::shell::HeapValue;

pub struct Frame {
    pub variables: HashMap<String, HeapValue>,
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            variables: HashMap::new(),
        }
    }
}