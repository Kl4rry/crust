use std::collections::HashMap;

use thin_string::ThinString;

use crate::{
    parser::{
        ast::{expr::Expr, Block, Variable},
        runtime_error::RunTimeError,
        P,
    },
    shell::values::{HeapValue, Value, ValueKind},
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

                for frame in shell.stack.iter_mut().rev() {
                    if let Some(heap_value) = frame.variables.get_mut(&var.name) {
                        *heap_value = value;
                        return Ok(());
                    }
                }

                shell
                    .stack
                    .last_mut()
                    .expect("stack is empty this should be impossible")
                    .variables
                    .insert(var.name.clone(), value);
            }
            Self::Declaration(var, expr) => {
                if let Some(expr) = expr {
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
                } else {
                    if !shell
                        .stack
                        .last_mut()
                        .expect("stack is empty this should be impossible")
                        .variables
                        .contains_key(&var.name)
                    {
                        shell
                            .stack
                            .last_mut()
                            .expect("stack is empty this should be impossible")
                            .variables
                            .insert(var.name.clone(), Value::String(ThinString::from("")).into());
                    }
                }
            }
            Self::Export(_var, _expr) => todo!("export not impl"),
            Self::If(expr, block, else_clause) => {
                let value = expr.eval(shell, false)?;
                if value.truthy() {
                    block.eval(shell, None)?
                } else {
                    if let Some(statement) = else_clause {
                        match &**statement {
                            Self::Block(block) => block.eval(shell, None)?,
                            Self::If(..) => statement.eval(shell)?,
                            _ => (),
                        }
                    }
                }
            }
            Self::Loop(block) => loop {
                match block.eval(shell, None) {
                    Ok(_) => (),
                    Err(RunTimeError::Break) => break,
                    Err(RunTimeError::Continue) => continue,
                    Err(error) => return Err(error),
                }
            },
            Self::While(condition, block) => {
                while condition.eval(shell, false)?.truthy() {
                    match block.eval(shell, None) {
                        Ok(_) => (),
                        Err(RunTimeError::Break) => break,
                        Err(RunTimeError::Continue) => continue,
                        Err(error) => return Err(error),
                    }
                }
            }
            Self::For(var, expr, block) => {
                let name = var.name.clone();
                let value: HeapValue = expr.eval(shell, false)?.into();
                match &*value {
                    Value::List(list) => {
                        for item in list.iter() {
                            let mut variables: HashMap<String, HeapValue> = HashMap::new();
                            variables.insert(name.clone(), item.clone());
                            match block.eval(shell, Some(variables)) {
                                Ok(_) => (),
                                Err(RunTimeError::Break) => break,
                                Err(RunTimeError::Continue) => continue,
                                Err(error) => return Err(error),
                            }
                        }
                    }
                    Value::String(string) => {
                        for c in string.chars() {
                            let mut variables: HashMap<String, HeapValue> = HashMap::new();
                            let item: HeapValue = Value::String(ThinString::from(c)).into();
                            variables.insert(name.clone(), item.clone());
                            match block.eval(shell, Some(variables)) {
                                Ok(_) => (),
                                Err(RunTimeError::Break) => break,
                                Err(RunTimeError::Continue) => continue,
                                Err(error) => return Err(error),
                            }
                        }
                    }
                    Value::Range(range) => {
                        for i in (**range).clone().into_iter() {
                            let mut variables: HashMap<String, HeapValue> = HashMap::new();
                            let item: HeapValue = Value::Int(i).into();
                            variables.insert(name.clone(), item.clone());
                            match block.eval(shell, Some(variables)) {
                                Ok(_) => (),
                                Err(RunTimeError::Break) => break,
                                Err(RunTimeError::Continue) => continue,
                                Err(error) => return Err(error),
                            }
                        }
                    }
                    _ => return Err(RunTimeError::InvalidIterator(value.to_type())),
                }
            }
            Self::Fn(name, params, block) => {
                shell
                    .stack
                    .last_mut()
                    .unwrap()
                    .functions
                    .insert(name.clone(), (params.clone(), block.clone()));
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
