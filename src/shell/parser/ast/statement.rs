use std::{collections::HashMap, rc::Rc};

use thin_string::ThinString;

use crate::{
    parser::{
        ast::{expr::Expr, Block, Variable},
        runtime_error::RunTimeError,
        P,
    },
    shell::{builtins::variables::is_builtin, value::Value},
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
    pub fn eval(&self, shell: &mut Shell) -> Result<(), RunTimeError> {
        match self {
            Self::Assign(var, expr) => {
                if is_builtin(&var.name) {
                    return Ok(());
                }

                let value = expr.eval(shell, false)?;

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
                if is_builtin(&var.name) {
                    return Ok(());
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
                        .insert(var.name.clone(), Value::String(ThinString::from("")));
                }
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
                    return Ok(());
                }

                for frame in shell.stack.iter_mut().rev() {
                    if let Some(heap_value) = frame.variables.get_mut(&var.name) {
                        *heap_value = res;
                        return Ok(());
                    }
                }

                shell
                    .stack
                    .last_mut()
                    .expect("stack is empty this should be impossible")
                    .variables
                    .insert(var.name.clone(), res);
            }
            Self::Export(_var, _expr) => todo!("export not impl"),
            Self::If(expr, block, else_clause) => {
                let value = expr.eval(shell, false)?;
                if value.truthy() {
                    block.eval(shell, None)?
                } else if let Some(statement) = else_clause {
                    match &**statement {
                        Self::Block(block) => block.eval(shell, None)?,
                        Self::If(..) => statement.eval(shell)?,
                        _ => unreachable!(),
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
                let value = expr.eval(shell, false)?;
                match value {
                    Value::List(list) => {
                        for item in list.iter() {
                            let mut variables: HashMap<String, Value> = HashMap::new();
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
                            let mut variables: HashMap<String, Value> = HashMap::new();
                            let item: Value = Value::String(ThinString::from(c));
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
                        for i in (*range).clone() {
                            let mut variables: HashMap<String, Value> = HashMap::new();
                            let item: Value = Value::Int(i);
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
            Self::Fn(name, func) => {
                shell
                    .stack
                    .last_mut()
                    .unwrap()
                    .functions
                    .insert(name.clone(), func.clone());
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
            Self::Block(block) => block.eval(shell, None)?,
        }
        Ok(())
    }
}
