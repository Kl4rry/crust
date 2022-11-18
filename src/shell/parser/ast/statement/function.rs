use std::sync::Arc;

use crate::parser::{
    ast::{variable::Variable, Block},
    source::Source,
};

#[derive(Debug)]
pub struct Function {
    pub parameters: Vec<Variable>,
    pub block: Block,
    pub src: Arc<Source>,
}
