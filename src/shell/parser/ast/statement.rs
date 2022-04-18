use std::{collections::HashMap, rc::Rc, sync::atomic::Ordering};

use crate::{
    parser::{
        ast::{expr::Expr, Block, Variable},
        shell_error::ShellErrorKind,
    },
    shell::{builtins::variables::is_builtin, stream::OutputStream, value::Value},
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
    pub fn eval(&self, shell: &mut Shell, output: &mut OutputStream) -> Result<(), ShellErrorKind> {
        match self {
            Self::Assign(var, expr) => {
                if is_builtin(&var.name) {
                    return Ok(());
                }

                let value = expr.eval(shell, output)?;

                for frame in shell.stack.iter_mut().rev() {
                    if let Some((_, heap_value)) = frame.variables.get_mut(&var.name) {
                        *heap_value = value;
                        return Ok(());
                    }
                }

                shell
                    .stack
                    .last_mut()
                    .expect("stack is empty this should be impossible")
                    .variables
                    .insert(var.name.clone(), (false, value));
                Ok(())
            }
            Self::Declaration(var, expr) => {
                if is_builtin(&var.name) {
                    // this should be a hard error
                    return Ok(());
                }

                let value = expr.eval(shell, output)?;
                shell
                    .stack
                    .last_mut()
                    .expect("stack is empty this should be impossible")
                    .variables
                    .insert(var.name.clone(), (false, value));

                Ok(())
            }
            Self::AssignOp(var, op, expr) => {
                if is_builtin(&var.name) {
                    // this should be a hard error
                    return Ok(());
                }

                let current = var.eval(shell)?;
                let res = match op {
                    AssignOp::Expo => current.try_expo(expr.eval(shell, output)?),
                    AssignOp::Add => current.try_add(expr.eval(shell, output)?),
                    AssignOp::Sub => current.try_sub(expr.eval(shell, output)?),
                    AssignOp::Mul => current.try_mul(expr.eval(shell, output)?),
                    AssignOp::Div => current.try_div(expr.eval(shell, output)?),
                    AssignOp::Mod => current.try_mod(expr.eval(shell, output)?),
                }?;

                for frame in shell.stack.iter_mut().rev() {
                    if let Some((_, heap_value)) = frame.variables.get_mut(&var.name) {
                        *heap_value = res;
                        return Ok(());
                    }
                }
                Ok(())
            }
            Self::Export(var, expr) => {
                if is_builtin(&var.name) {
                    // this should be a hard error
                    return Ok(());
                }

                let value = expr.eval(shell, output)?;
                if !matches!(
                    &value,
                    Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_)
                ) {
                    return Err(ShellErrorKind::InvalidEnvVar(value.to_type()));
                }

                shell
                    .stack
                    .last_mut()
                    .expect("stack is empty this should be impossible")
                    .variables
                    .insert(var.name.clone(), (true, value));
                Ok(())
            }
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

                fn for_loop(
                    shell: &mut Shell,
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
                        variables.insert(name.to_string(), (false, item.clone()));
                        match block.eval(shell, Some(variables), None, output) {
                            Ok(()) => (),
                            Err(ShellErrorKind::Break) => break,
                            Err(ShellErrorKind::Continue) => continue,
                            Err(error) => return Err(error),
                        }
                    }
                    Ok(())
                }

                match value {
                    Value::List(list) => for_loop(shell, list.into_iter(), &name, block, output),
                    Value::String(string) => for_loop(
                        shell,
                        string.chars().map(|c| Value::String(String::from(c))),
                        &name,
                        block,
                        output,
                    ),
                    Value::Range(range) =>
                    {
                        #[allow(clippy::redundant_closure)]
                        for_loop(shell, range.map(|i| Value::Int(i)), &name, block, output)
                    }
                    _ => Err(ShellErrorKind::InvalidIterator(value.to_type())),
                }
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
