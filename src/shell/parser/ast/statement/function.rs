use std::{rc::Rc, sync::Arc};

use miette::NamedSource;

use crate::parser::{
    ast::{variable::Variable, Block},
    lexer::token::span::Span,
};

#[derive(Debug)]
pub struct Function {
    pub name: Rc<str>,
    pub arg_span: Span,
    pub parameters: Vec<Variable>,
    pub block: Block,
    pub src: Arc<NamedSource<String>>,
}
