use std::collections::HashMap;

use crate::{parser::ast::{Block, variable::Variable}, shell::HeapValue};

pub struct Frame {
    pub variables: HashMap<String, HeapValue>,
    pub functions: HashMap<String, (Vec<Variable>, Block)>,
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }
}
