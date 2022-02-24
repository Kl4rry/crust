use std::{collections::HashMap, rc::Rc, sync::atomic::Ordering};

use crate::{
    parser::{
        ast::{expr::Expr, Block, Variable},
        shell_error::ShellErrorKind,
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
    pub fn eval(&self, shell: &mut Shell, output: &mut OutputStream) -> Result<(), ShellErrorKind> {
        match self {
            Self::Assign(var, expr) => {
                if is_builtin(&var.name) {
                    return Ok(());
                }

                let value = expr.eval(shell, output)?;

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
                Ok(())
            }
            Self::Declaration(var, expr) => {
                if is_builtin(&var.name) {
                    // this should be a hard error
                    return Ok(());
                }

                if let Some(expr) = expr {
                    let value = expr.eval(shell, output)?;
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
                Ok(())
            }
            Self::AssignOp(var, op, expr) => {
                let current = var.eval(shell)?;
                let res = match op {
                    AssignOp::Expo => current.try_expo(expr.eval(shell, output)?),
                    AssignOp::Add => current.try_add(expr.eval(shell, output)?),
                    AssignOp::Sub => current.try_sub(expr.eval(shell, output)?),
                    AssignOp::Mul => current.try_mul(expr.eval(shell, output)?),
                    AssignOp::Div => current.try_div(expr.eval(shell, output)?),
                    AssignOp::Mod => current.try_mod(expr.eval(shell, output)?),
                }?;

                if is_builtin(&var.name) {
                    // this should be a hard error
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
                Ok(())
            }
            Self::Export(_var, _expr) => todo!("export not impl"),
            Self::If(expr, block, else_clause) => {
                let value = expr.eval(shell, output)?;
                if value.truthy() {
                    block.eval(shell, None, None, output)?
                } else if let Some(statement) = else_clause {
                    match &**statement {
                        Self::Block(block) => block.eval(shell, None, None, output)?,
                        Self::If(..) => statement.eval(shell, output)?,
                        _ => unreachable!(),
                    }
                }
                Ok(())
            }
            Self::Loop(block) => loop {
                if shell.interrupt.load(Ordering::SeqCst) {
                    return Err(ShellErrorKind::Interrupt);
                }
                match block.eval(shell, None, None, output) {
                    Ok(()) => (),
                    Err(ShellErrorKind::Break) => return Ok(()),
                    Err(ShellErrorKind::Continue) => continue,
                    Err(error) => return Err(error),
                }
            },
            Self::While(condition, block) => {
                while condition.eval(shell, output)?.truthy() {
                    if shell.interrupt.load(Ordering::SeqCst) {
                        return Err(ShellErrorKind::Interrupt);
                    }

                    match block.eval(shell, None, None, output) {
                        Ok(()) => (),
                        Err(ShellErrorKind::Break) => break,
                        Err(ShellErrorKind::Continue) => continue,
                        Err(error) => return Err(error),
                    }
                }
                Ok(())
            }
            Self::For(var, expr, block) => {
                let name = var.name.clone();
                let value = expr.eval(shell, output)?;
                match value {
                    Value::List(list) => {
                        for item in list.iter() {
                            if shell.interrupt.load(Ordering::SeqCst) {
                                return Err(ShellErrorKind::Interrupt);
                            }

                            let mut variables: HashMap<String, Value> = HashMap::new();
                            variables.insert(name.clone(), item.clone());
                            match block.eval(shell, Some(variables), None, output) {
                                Ok(()) => (),
                                Err(ShellErrorKind::Break) => break,
                                Err(ShellErrorKind::Continue) => continue,
                                Err(error) => return Err(error),
                            }
                        }
                    }
                    Value::String(string) => {
                        for c in string.chars() {
                            if shell.interrupt.load(Ordering::SeqCst) {
                                return Err(ShellErrorKind::Interrupt);
                            }

                            let mut variables: HashMap<String, Value> = HashMap::new();
                            let item: Value = Value::String(String::from(c));
                            variables.insert(name.clone(), item.clone());
                            match block.eval(shell, Some(variables), None, output) {
                                Ok(()) => (),
                                Err(ShellErrorKind::Break) => break,
                                Err(ShellErrorKind::Continue) => continue,
                                Err(error) => return Err(error),
                            }
                        }
                    }
                    Value::Range(range) => {
                        for i in (*range).clone() {
                            if shell.interrupt.load(Ordering::SeqCst) {
                                return Err(ShellErrorKind::Interrupt);
                            }

                            let mut variables: HashMap<String, Value> = HashMap::new();
                            let item: Value = Value::Int(i);
                            variables.insert(name.clone(), item.clone());
                            match block.eval(shell, Some(variables), None, output) {
                                Ok(()) => (),
                                Err(ShellErrorKind::Break) => break,
                                Err(ShellErrorKind::Continue) => continue,
                                Err(error) => return Err(error),
                            }
                        }
                    }
                    _ => return Err(ShellErrorKind::InvalidIterator(value.to_type())),
                }
                Ok(())
            }
            Self::Fn(name, func) => {
                if name == "prompt" && func.0.is_empty() {
                    shell.prompt = Some(func.1.clone());
                } else {
                    shell
                        .stack
                        .last_mut()
                        .unwrap()
                        .functions
                        .insert(name.clone(), func.clone());
                }
                Ok(())
            }
            Self::Return(expr) => {
                if let Some(expr) = expr {
                    let value = expr.eval(shell, output)?;
                    Err(ShellErrorKind::Return(Some(value)))
                } else {
                    Err(ShellErrorKind::Return(None))
                }
            }
            Self::Break => Err(ShellErrorKind::Break),
            Self::Continue => Err(ShellErrorKind::Continue),
            Self::Block(block) => block.eval(shell, None, None, output),
        }
    }
}
