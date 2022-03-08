use std::{collections::HashMap, rc::Rc};

use super::stream::ValueStream;
use crate::{
    parser::ast::{variable::Variable, Block},
    shell::value::Value,
};

#[derive(Debug, Default)]
pub struct Frame {
    pub variables: HashMap<String, (bool, Value)>,
    pub functions: HashMap<String, Rc<(Vec<Variable>, Block)>>,
    pub input: ValueStream,
}

impl Frame {
    pub fn new(
        variables: HashMap<String, (bool, Value)>,
        functions: HashMap<String, Rc<(Vec<Variable>, Block)>>,
        input: ValueStream,
    ) -> Self {
        Self {
            variables,
            functions,
            input,
        }
    }
}
