use std::{collections::HashMap, rc::Rc, sync::atomic::Ordering};

use crate::{
    parser::{
        ast::{expr::Expr, Block, Variable},
        shell_error::ShellErrorKind,
    },
    shell::{builtins::variables::is_builtin, frame::Frame, stream::OutputStream, value::Value},
    Shell, P,
};

pub mod assign_op;
use assign_op::AssignOp;

#[derive(Debug, Clone)]
pub enum Statement {
    Export(Variable, Expr),
    Declaration(Variable, Expr),
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
    pub fn eval(
        &self,
        shell: &mut Shell,
        frame: &mut Frame,
        output: &mut OutputStream,
    ) -> Result<(), ShellErrorKind> {
        match self {
            Self::Assign(var, expr) => {
                if is_builtin(&var.name) {
                    return Err(ShellErrorKind::ReadOnlyVar(var.name.clone()));
                }

                let value = expr.eval(shell, frame, output)?;
                if let Some(value) = frame.update_var(&var.name, value)? {
                    frame.add_var(var.name.clone(), value);
                }
                Ok(())
            }
            Self::Declaration(var, expr) => {
                if is_builtin(&var.name) {
                    return Err(ShellErrorKind::ReadOnlyVar(var.name.to_string()));
                }

                let value = expr.eval(shell, frame, output)?;
                frame.add_var(var.name.clone(), value);

                Ok(())
            }
            Self::AssignOp(var, op, expr) => {
                if is_builtin(&var.name) {
                    return Err(ShellErrorKind::ReadOnlyVar(var.name.to_string()));
                }

                let current = var.eval(shell, frame)?;
                let res = match op {
                    AssignOp::Expo => current.try_expo(expr.eval(shell, frame, output)?),
                    AssignOp::Add => current.try_add(expr.eval(shell, frame, output)?),
                    AssignOp::Sub => current.try_sub(expr.eval(shell, frame, output)?),
                    AssignOp::Mul => current.try_mul(expr.eval(shell, frame, output)?),
                    AssignOp::Div => current.try_div(expr.eval(shell, frame, output)?),
                    AssignOp::Mod => current.try_mod(expr.eval(shell, frame, output)?),
                }?;

                frame.update_var(&var.name, res)?;
                Ok(())
            }
            Self::Export(var, expr) => {
                if is_builtin(&var.name) {
                    return Err(ShellErrorKind::ReadOnlyVar(var.name.to_string()));
                }

                let value = expr.eval(shell, frame, output)?;
                if !matches!(
                    &value,
                    Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_)
                ) {
                    return Err(ShellErrorKind::InvalidEnvVar(value.to_type()));
                }

                frame.add_env_var(var.name.clone(), value);
                Ok(())
            }
            Self::If(expr, block, else_clause) => {
                let value = expr.eval(shell, frame, output)?;
                if value.truthy() {
                    block.eval(shell, frame.clone(), None, None, output)?
                } else if let Some(statement) = else_clause {
                    match &**statement {
                        Self::Block(block) => {
                            block.eval(shell, frame.clone(), None, None, output)?
                        }
                        Self::If(..) => statement.eval(shell, frame, output)?,
                        _ => unreachable!(),
                    }
                }
                Ok(())
            }
            Self::Loop(block) => loop {
                if shell.interrupt.load(Ordering::SeqCst) {
                    return Err(ShellErrorKind::Interrupt);
                }
                match block.eval(shell, frame.clone(), None, None, output) {
                    Ok(()) => (),
                    Err(ShellErrorKind::Break) => return Ok(()),
                    Err(ShellErrorKind::Continue) => continue,
                    Err(error) => return Err(error),
                }
            },
            Self::While(condition, block) => {
                while condition.eval(shell, frame, output)?.truthy() {
                    if shell.interrupt.load(Ordering::SeqCst) {
                        return Err(ShellErrorKind::Interrupt);
                    }

                    match block.eval(shell, frame.clone(), None, None, output) {
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
                let value = expr.eval(shell, frame, output)?;

                fn for_loop(
                    shell: &mut Shell,
                    frame: &mut Frame,
                    iterator: impl Iterator<Item = Value>,
                    name: &str,
                    block: &Block,
                    output: &mut OutputStream,
                ) -> Result<(), ShellErrorKind> {
                    for item in iterator {
                        if shell.interrupt.load(Ordering::SeqCst) {
                            return Err(ShellErrorKind::Interrupt);
                        }

                        let mut variables: HashMap<String, (bool, Value)> = HashMap::new();
                        variables.insert(name.to_string(), (false, item.to_owned()));
                        match block.eval(shell, frame.clone(), Some(variables), None, output) {
                            Ok(()) => (),
                            Err(ShellErrorKind::Break) => break,
                            Err(ShellErrorKind::Continue) => continue,
                            Err(error) => return Err(error),
                        }
                    }
                    Ok(())
                }

                match value {
                    Value::List(list) => {
                        for_loop(shell, frame, list.iter().cloned(), &name, block, output)
                    }
                    Value::String(string) => for_loop(
                        shell,
                        frame,
                        string.chars().map(|c| Value::from(String::from(c))),
                        &name,
                        block,
                        output,
                    ),
                    Value::Range(range) =>
                    {
                        #[allow(clippy::redundant_closure)]
                        for_loop(
                            shell,
                            frame,
                            Rc::unwrap_or_clone(range).map(|i| Value::Int(i)),
                            &name,
                            block,
                            output,
                        )
                    }
                    Value::Map(map) => for_loop(
                        shell,
                        frame,
                        map.iter()
                            .map(|(k, v)| Value::from(vec![Value::from(k.to_string()), v.clone()])),
                        &name,
                        block,
                        output,
                    ),
                    Value::Table(table) => for_loop(
                        shell,
                        frame,
                        table.iter().map(Value::from),
                        &name,
                        block,
                        output,
                    ),
                    _ => Err(ShellErrorKind::InvalidIterator(value.to_type())),
                }
            }
            Self::Fn(name, func) => {
                if name == "prompt" && func.0.is_empty() {
                    shell.prompt = Some(func.1.clone());
                } else {
                    frame.add_function(name.clone(), func.clone());
                }
                Ok(())
            }
            Self::Return(expr) => {
                if let Some(expr) = expr {
                    let value = expr.eval(shell, frame, output)?;
                    Err(ShellErrorKind::Return(Some(value)))
                } else {
                    Err(ShellErrorKind::Return(None))
                }
            }
            Self::Break => Err(ShellErrorKind::Break),
            Self::Continue => Err(ShellErrorKind::Continue),
            Self::Block(block) => block.eval(shell, frame.clone(), None, None, output),
        }
    }
}
