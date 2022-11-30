use std::{collections::HashMap, rc::Rc, sync::atomic::Ordering};

use crate::{
    parser::{
        ast::{expr::Expr, Block, Variable},
        lexer::token::span::Span,
        shell_error::ShellErrorKind,
    },
    shell::{builtins::variables::is_builtin, value::Value},
    P,
};

pub mod assign_op;
use assign_op::AssignOp;

pub mod function;

use self::function::Function;
use super::context::Context;

#[derive(Clone, Debug)]
pub enum StatementKind {
    Export(Variable, Expr),
    Declaration(Variable, Expr),
    Assign(Variable, Expr),
    AssignOp(Variable, AssignOp, Expr),
    If(Expr, Block, Option<P<Statement>>),
    Fn(String, Rc<Function>),
    Return(Option<Expr>),
    For(Variable, Expr, Block),
    While(Expr, Block),
    Loop(Block),
    Block(Block),
    Continue,
    Break,
}

impl StatementKind {
    pub fn spanned(self, span: Span) -> Statement {
        Statement { kind: self, span }
    }
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub kind: StatementKind,
    pub span: Span,
}

impl Statement {
    pub fn eval(&self, ctx: &mut Context) -> Result<(), ShellErrorKind> {
        match &self.kind {
            StatementKind::Assign(var, expr) => {
                if is_builtin(&var.name) {
                    return Err(ShellErrorKind::ReadOnlyVar(var.name.clone()));
                }

                let value = expr.eval(ctx)?;
                if let Some(value) = ctx.frame.update_var(&var.name, value)? {
                    ctx.frame.add_var(var.name.clone(), value);
                }
                Ok(())
            }
            StatementKind::Declaration(var, expr) => {
                if is_builtin(&var.name) {
                    return Err(ShellErrorKind::ReadOnlyVar(var.name.to_string()));
                }

                let value = expr.eval(ctx)?;
                ctx.frame.add_var(var.name.clone(), value);

                Ok(())
            }
            StatementKind::AssignOp(var, op, expr) => {
                if is_builtin(&var.name) {
                    return Err(ShellErrorKind::ReadOnlyVar(var.name.to_string()));
                }

                let current = var.eval(ctx)?;
                let res = match op {
                    AssignOp::Expo => current.try_expo(expr.eval(ctx)?),
                    AssignOp::Add => current.try_add(expr.eval(ctx)?),
                    AssignOp::Sub => current.try_sub(expr.eval(ctx)?),
                    AssignOp::Mul => current.try_mul(expr.eval(ctx)?),
                    AssignOp::Div => current.try_div(expr.eval(ctx)?),
                    AssignOp::Mod => current.try_mod(expr.eval(ctx)?),
                }?;

                ctx.frame.update_var(&var.name, res)?;
                Ok(())
            }
            StatementKind::Export(var, expr) => {
                if is_builtin(&var.name) {
                    return Err(ShellErrorKind::ReadOnlyVar(var.name.to_string()));
                }

                let value = expr.eval(ctx)?;
                if !matches!(
                    &value,
                    Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_)
                ) {
                    return Err(ShellErrorKind::InvalidEnvVar(value.to_type()));
                }

                ctx.frame.add_env_var(var.name.clone(), value);
                Ok(())
            }
            StatementKind::If(expr, block, else_clause) => {
                let value = expr.eval(ctx)?;
                if value.truthy() {
                    block.eval(ctx, None, None)?
                } else if let Some(statement) = else_clause {
                    match &statement.kind {
                        StatementKind::Block(block) => block.eval(ctx, None, None)?,
                        StatementKind::If(..) => statement.eval(ctx)?,
                        _ => unreachable!(),
                    }
                }
                Ok(())
            }
            StatementKind::Loop(block) => loop {
                if ctx.shell.interrupt.load(Ordering::SeqCst) {
                    return Err(ShellErrorKind::Interrupt);
                }
                match block.eval(ctx, None, None) {
                    Ok(()) => (),
                    Err(ShellErrorKind::Break) => return Ok(()),
                    Err(ShellErrorKind::Continue) => continue,
                    Err(error) => return Err(error),
                }
            },
            StatementKind::While(condition, block) => {
                while condition.eval(ctx)?.truthy() {
                    if ctx.shell.interrupt.load(Ordering::SeqCst) {
                        return Err(ShellErrorKind::Interrupt);
                    }

                    match block.eval(ctx, None, None) {
                        Ok(()) => (),
                        Err(ShellErrorKind::Break) => break,
                        Err(ShellErrorKind::Continue) => continue,
                        Err(error) => return Err(error),
                    }
                }
                Ok(())
            }
            StatementKind::For(var, expr, block) => {
                let name = var.name.clone();
                let value = expr.eval(ctx)?;

                fn for_loop(
                    ctx: &mut Context,
                    iterator: impl Iterator<Item = Value>,
                    name: &str,
                    block: &Block,
                ) -> Result<(), ShellErrorKind> {
                    for item in iterator {
                        if ctx.shell.interrupt.load(Ordering::SeqCst) {
                            return Err(ShellErrorKind::Interrupt);
                        }

                        let mut variables: HashMap<String, (bool, Value)> = HashMap::new();
                        variables.insert(name.to_string(), (false, item.to_owned()));
                        match block.eval(ctx, Some(variables), None) {
                            Ok(()) => (),
                            Err(ShellErrorKind::Break) => break,
                            Err(ShellErrorKind::Continue) => continue,
                            Err(error) => return Err(error),
                        }
                    }
                    Ok(())
                }

                match value {
                    Value::List(list) => for_loop(ctx, list.iter().cloned(), &name, block),
                    Value::String(string) => for_loop(
                        ctx,
                        string.chars().map(|c| Value::from(String::from(c))),
                        &name,
                        block,
                    ),
                    Value::Range(range) =>
                    {
                        #[allow(clippy::redundant_closure)]
                        for_loop(
                            ctx,
                            Rc::unwrap_or_clone(range).map(|i| Value::Int(i)),
                            &name,
                            block,
                        )
                    }
                    Value::Map(map) => for_loop(
                        ctx,
                        map.iter()
                            .map(|(k, v)| Value::from(vec![Value::from(k.to_string()), v.clone()])),
                        &name,
                        block,
                    ),
                    Value::Table(table) => {
                        for_loop(ctx, table.iter().map(Value::from), &name, block)
                    }
                    _ => Err(ShellErrorKind::InvalidIterator(value.to_type())),
                }
            }
            StatementKind::Fn(name, func) => {
                ctx.frame.add_function(name.clone(), func.clone());
                Ok(())
            }
            StatementKind::Return(expr) => {
                if let Some(expr) = expr {
                    let value = expr.eval(ctx)?;
                    Err(ShellErrorKind::Return(Some(value)))
                } else {
                    Err(ShellErrorKind::Return(None))
                }
            }
            StatementKind::Break => Err(ShellErrorKind::Break),
            StatementKind::Continue => Err(ShellErrorKind::Continue),
            StatementKind::Block(block) => block.eval(ctx, None, None),
        }
    }
}
