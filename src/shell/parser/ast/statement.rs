use crate::{
    parser::{
        ast::{
            expr::{argument::Argument, Expr},
            Block, Variable,
        },
        runtime_error::RunTimeError,
        P,
    },
    Shell,
};

#[derive(Debug)]
pub enum Statement {
    Export(Variable, Option<Expr>),
    Declaration(Variable, Option<Expr>),
    Assignment(Variable, Expr),
    Alias(Argument, Expr),
    If(Expr, Block, Option<P<Statement>>),
    Fn(String, Vec<Variable>, Block),
    Return(Option<Expr>),
    Loop(Block),
    While(Expr, Block),
    For(Variable, Expr, Block),
    Break,
    Continue,
    Block(Block),
}

impl Statement {
    pub fn eval(&self, _shell: &mut Shell) -> Result<(), RunTimeError> {
        Ok(())
    }
}
