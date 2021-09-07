use crate::{
    parser::{
        ast::{expr::Expr, Block, Variable},
        runtime_error::RunTimeError,
        P,
    },
    shell::gc::ValueKind,
    Shell,
};

#[derive(Debug)]
pub enum Statement {
    Export(Variable, Option<Expr>),
    Declaration(Variable, Option<Expr>),
    Assignment(Variable, Expr),
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
    pub fn eval(&self, shell: &mut Shell) -> Result<(), RunTimeError> {
        match self {
            Self::Assignment(var, expr) => {
                let value = match expr.eval(shell, false)? {
                    ValueKind::Heap(value) => value,
                    ValueKind::Stack(value) => value.into(),
                };
                shell
                    .variable_stack
                    .last_mut()
                    .expect("stack is empty this should be impossible")
                    .insert(var.name.clone(), value);
            }
            Self::Export(_var, _expr) => {}
            _ => todo!("statement not impl"),
        }
        Ok(())
    }
}
