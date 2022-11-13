use std::sync::Arc;

use crate::parser::{
    ast::{variable::Variable, Block},
    source::Source,
};

pub struct Function {
    pub parameters: Vec<Variable>,
    pub block: Block,
    pub src: Arc<Source>,
}
