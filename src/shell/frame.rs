use std::collections::HashMap;

use crate::{
    parser::ast::{variable::Variable, Block},
    shell::value::Value,
};

pub struct Frame {
    pub variables: HashMap<String, Value>,
    pub functions: HashMap<String, (Vec<Variable>, Block)>,
}

impl Frame {
    pub fn new() -> Self {
        Frame {
            variables: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn with_variables(variables: HashMap<String, Value>) -> Self {
        Frame {
            variables,
            functions: HashMap::new(),
        }
    }
}
