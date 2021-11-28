use std::collections::HashMap;

use crate::{
    parser::ast::{variable::Variable, Block},
    shell::HeapValue,
};

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

    pub fn with_variables(variables: HashMap<String, HeapValue>) -> Self {
        Frame {
            variables,
            functions: HashMap::new(),
        }
    }
}
