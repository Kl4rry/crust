use std::{collections::HashMap, rc::Rc, sync::atomic::Ordering};

use crate::{
    parser::{
        ast::{expr::Expr, Block, Variable},
        runtime_error::RunTimeError,
        P,
    },
    shell::{builtins::variables::is_builtin, stream::OutputStream, value::Value},
    Shell,
};

pub mod assign_op;
use assign_op::AssignOp;

#[derive(Debug, Clone)]
pub enum Statement {
    Export(Variable, Option<Expr>),
    Declaration(Variable, Option<Expr>),
    Assign(Variable, Expr),
    AssignOp(Variable, AssignOp, Expr),
    If(Expr, Block, Option<P<Statement>>),
    Fn(String, Rc<(Vec<Variable>, Block)>),
    Return(Option<Expr>),
    For(Variable, Expr, Block),
    While(Expr, Block),
    Loop(Block),
    Block(Block),
    Continue,
    Break,
}

impl Statement {
    pub fn eval(&self, shell: &mut Shell) -> Result<Value, RunTimeError> {
        match self {
            Self::Assign(var, expr) => {
                if is_builtin(&var.name) {
                    return Ok(Value::Null);
                }

                let value = expr.eval(shell, false)?;

                for frame in shell.stack.iter_mut().rev() {
                    if let Some(heap_value) = frame.variables.get_mut(&var.name) {
                        *heap_value = value;
                        return Ok(Value::Null);
                    }
                }

                shell
                    .stack
                    .last_mut()
                    .expect("stack is empty this should be impossible")
                    .variables
                    .insert(var.name.clone(), value);
                Ok(Value::Null)
            }
            Self::Declaration(var, expr) => {
                if is_builtin(&var.name) {
                    // this should be a hard error
                    return Ok(Value::Null);
                }

                if let Some(expr) = expr {
                    let value = expr.eval(shell, false)?;
                    shell
                        .stack
                        .last_mut()
                        .expect("stack is empty this should be impossible")
                        .variables
                        .insert(var.name.clone(), value);
                } else if !shell
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
                        .insert(var.name.clone(), Value::String(String::from("")));
                }
                Ok(Value::Null)
            }
            Self::AssignOp(var, op, expr) => {
                let current = var.eval(shell)?;
                let res = match op {
                    AssignOp::Expo => current.try_expo(expr.eval(shell, false)?),
                    AssignOp::Add => current.try_add(expr.eval(shell, false)?),
                    AssignOp::Sub => current.try_sub(expr.eval(shell, false)?),
                    AssignOp::Mul => current.try_mul(expr.eval(shell, false)?),
                    AssignOp::Div => current.try_div(expr.eval(shell, false)?),
                    AssignOp::Mod => current.try_mod(expr.eval(shell, false)?),
                }?;

                if is_builtin(&var.name) {
                    // this should be a hard error
                    return Ok(Value::Null);
                }

                for frame in shell.stack.iter_mut().rev() {
                    if let Some(heap_value) = frame.variables.get_mut(&var.name) {
                        *heap_value = res;
                        return Ok(Value::Null);
                    }
                }

                shell
                    .stack
                    .last_mut()
                    .expect("stack is empty this should be impossible")
                    .variables
                    .insert(var.name.clone(), res);
                Ok(Value::Null)
            }
            Self::Export(_var, _expr) => todo!("export not impl"),
            Self::If(expr, block, else_clause) => {
                let value = expr.eval(shell, false)?;
                if value.truthy() {
                    Ok(Value::OutputStream(P::new(block.eval(shell, None, None)?)))
                } else if let Some(statement) = else_clause {
                    match &**statement {
                        Self::Block(block) => {
                            Ok(Value::OutputStream(P::new(block.eval(shell, None, None)?)))
                        }
                        Self::If(..) => Ok(statement.eval(shell)?),
                        _ => unreachable!(),
                    }
                } else {
                    Ok(Value::Null)
                }
            }
            Self::Loop(block) => loop {
                if shell.interrupt.load(Ordering::SeqCst) {
                    return Err(RunTimeError::Interrupt);
                }
                let mut collection = OutputStream::default();
                match block.eval(shell, None, None) {
                    Ok(stream) => collection.extend(stream.into_iter()),
                    Err(RunTimeError::Break) => return Ok(Value::OutputStream(P::new(collection))),
                    Err(RunTimeError::Continue) => continue,
                    Err(error) => return Err(error),
                }
            },
            Self::While(condition, block) => {
                let mut collection = OutputStream::default();
                while condition.eval(shell, false)?.truthy() {
                    if shell.interrupt.load(Ordering::SeqCst) {
                        return Err(RunTimeError::Interrupt);
                    }

                    match block.eval(shell, None, None) {
                        Ok(stream) => collection.extend(stream.into_iter()),
                        Err(RunTimeError::Break) => break,
                        Err(RunTimeError::Continue) => continue,
                        Err(error) => return Err(error),
                    }
                }
                Ok(Value::OutputStream(P::new(collection)))
            }
            Self::For(var, expr, block) => {
                let name = var.name.clone();
                let value = expr.eval(shell, false)?;
                let mut collection = OutputStream::default();
                match value {
                    Value::List(list) => {
                        for item in list.iter() {
                            if shell.interrupt.load(Ordering::SeqCst) {
                                return Err(RunTimeError::Interrupt);
                            }

                            let mut variables: HashMap<String, Value> = HashMap::new();
                            variables.insert(name.clone(), item.clone());
                            match block.eval(shell, Some(variables), None) {
                                Ok(stream) => collection.extend(stream.into_iter()),
                                Err(RunTimeError::Break) => break,
                                Err(RunTimeError::Continue) => continue,
                                Err(error) => return Err(error),
                            }
                        }
                    }
                    Value::String(string) => {
                        for c in string.chars() {
                            if shell.interrupt.load(Ordering::SeqCst) {
                                return Err(RunTimeError::Interrupt);
                            }

                            let mut variables: HashMap<String, Value> = HashMap::new();
                            let item: Value = Value::String(String::from(c));
                            variables.insert(name.clone(), item.clone());
                            match block.eval(shell, Some(variables), None) {
                                Ok(stream) => collection.extend(stream.into_iter()),
                                Err(RunTimeError::Break) => break,
                                Err(RunTimeError::Continue) => continue,
                                Err(error) => return Err(error),
                            }
                        }
                    }
                    Value::Range(range) => {
                        for i in (*range).clone() {
                            if shell.interrupt.load(Ordering::SeqCst) {
                                return Err(RunTimeError::Interrupt);
                            }

                            let mut variables: HashMap<String, Value> = HashMap::new();
                            let item: Value = Value::Int(i);
                            variables.insert(name.clone(), item.clone());
                            match block.eval(shell, Some(variables), None) {
                                Ok(stream) => collection.extend(stream.into_iter()),
                                Err(RunTimeError::Break) => break,
                                Err(RunTimeError::Continue) => continue,
                                Err(error) => return Err(error),
                            }
                        }
                    }
                    _ => return Err(RunTimeError::InvalidIterator(value.to_type())),
                }
                Ok(Value::OutputStream(P::new(collection)))
            }
            Self::Fn(name, func) => {
                shell
                    .stack
                    .last_mut()
                    .unwrap()
                    .functions
                    .insert(name.clone(), func.clone());
                Ok(Value::Null)
            }
            Self::Return(expr) => {
                if let Some(expr) = expr {
                    let value = expr.eval(shell, false)?;
                    Err(RunTimeError::Return(Some(value)))
                } else {
                    Err(RunTimeError::Return(None))
                }
            }
            Self::Break => Err(RunTimeError::Break),
            Self::Continue => Err(RunTimeError::Continue),
            Self::Block(block) => Ok(Value::OutputStream(P::new(block.eval(shell, None, None)?))),
        }
    }
}
