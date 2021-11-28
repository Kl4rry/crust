use crate::{
    parser::{
        ast::{expr::Expr, Block, Variable},
        runtime_error::RunTimeError,
        P,
    },
    shell::values::ValueKind,
    Shell,
};

#[derive(Debug, Clone)]
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
                    .stack
                    .last_mut()
                    .expect("stack is empty this should be impossible")
                    .variables
                    .insert(var.name.clone(), value);
            }
            Self::Export(_var, _expr) => todo!("export not impl"),
            Self::If(expr, block, else_clause) => {
                let value = expr.eval(shell, false)?;
                if value.truthy() {
                    block.eval(shell)?
                } else {
                    if let Some(statement) = else_clause {
                        match &**statement {
                            Self::Block(block) => block.eval(shell)?,
                            Self::If(..) => statement.eval(shell)?,
                            _ => (),
                        }
                    }
                }
            }
            Self::Loop(block) => loop {
                match block.eval(shell) {
                    Ok(_) => (),
                    Err(RunTimeError::Break) => break,
                    Err(RunTimeError::Continue) => continue,
                    Err(error) => return Err(error),
                }
            },
            Self::While(condition, block) => {
                while condition.eval(shell, false)?.truthy() {
                    match block.eval(shell) {
                        Ok(_) => (),
                        Err(RunTimeError::Break) => break,
                        Err(RunTimeError::Continue) => continue,
                        Err(error) => return Err(error),
                    }
                }
            }
            Self::Fn(name, params, block) => {
                shell.stack.last_mut().unwrap().functions.insert(name.clone(), (params.clone(), block.clone()));
            }
            Self::Return(expr) => {
                if let Some(expr) = expr {
                    let value = expr.eval(shell, false)?;
                    return Err(RunTimeError::Return(Some(value)));
                } else {
                    return Err(RunTimeError::Return(None));
                }
            }
            Self::Break => {
                return Err(RunTimeError::Break);
            }
            Self::Continue => {
                return Err(RunTimeError::Continue);
            }
            statement => todo!("statement not impl: {:?}", statement),
        }
        Ok(())
    }
}
