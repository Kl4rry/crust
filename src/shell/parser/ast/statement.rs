use std::{collections::HashMap, rc::Rc, sync::atomic::Ordering};

use crate::{
    parser::{
        ast::{expr::Expr, statement::assign_op::AssignOpKind, Block, Variable},
        lexer::token::span::Span,
        shell_error::ShellErrorKind,
    },
    shell::{
        builtins::variables::{is_builtin, set_var, SetResult},
        value::Value,
    },
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
    Fn(Rc<str>, Rc<Function>),
    Return(Option<Expr>),
    For(Variable, Expr, Block),
    While(Expr, Block),
    Loop(Block),
    TryCatch(Block, Block),
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
                let value = expr.eval(ctx)?;
                let value = match set_var(ctx, &var.name, var.span, value) {
                    SetResult::Success => return Ok(()),
                    SetResult::NotFound(value) => value,
                    SetResult::Error(err) => return Err(err),
                };

                if let Some(value) = ctx.frame.update_var(&var.name, value.into())? {
                    ctx.frame.add_var(var.name.clone(), value);
                }
                Ok(())
            }
            StatementKind::AssignOp(var, op, expr) => {
                let current = var.eval(ctx)?;
                let res = match op.kind {
                    AssignOpKind::Expo => current.try_expo(expr.eval(ctx)?, op.span),
                    AssignOpKind::Add => current.try_add(expr.eval(ctx)?, op.span),
                    AssignOpKind::Sub => current.try_sub(expr.eval(ctx)?, op.span),
                    AssignOpKind::Mul => current.try_mul(expr.eval(ctx)?, op.span),
                    AssignOpKind::Div => current.try_div(expr.eval(ctx)?, op.span),
                    AssignOpKind::Mod => current.try_mod(expr.eval(ctx)?, op.span),
                }?;

                ctx.frame.update_var(&var.name, res.value)?;
                Ok(())
            }
            StatementKind::Declaration(var, expr) => {
                if is_builtin(&var.name) {
                    return Err(ShellErrorKind::OverrideBuiltin(
                        var.name.to_string(),
                        var.span,
                    ));
                }

                let value = expr.eval(ctx)?;
                ctx.frame.add_var(var.name.clone(), value.into());
                Ok(())
            }
            StatementKind::Export(var, expr) => {
                if is_builtin(&var.name) {
                    return Err(ShellErrorKind::OverrideBuiltin(
                        var.name.to_string(),
                        var.span,
                    ));
                }

                let value = expr.eval(ctx)?;
                if !matches!(
                    &value.value,
                    Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::String(_)
                ) {
                    return Err(ShellErrorKind::InvalidEnvVar(value.value.to_type()));
                }

                ctx.frame.add_env_var(var.name.clone(), value.into());
                Ok(())
            }
            StatementKind::If(expr, block, else_clause) => {
                let value = expr.eval(ctx)?;
                if value.value.truthy() {
                    block.eval(ctx, None)?
                } else if let Some(statement) = else_clause {
                    match &statement.kind {
                        StatementKind::Block(block) => block.eval(ctx, None)?,
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
                match block.eval(ctx, None) {
                    Ok(()) => (),
                    Err(ShellErrorKind::Break) => return Ok(()),
                    Err(ShellErrorKind::Continue) => continue,
                    Err(error) => return Err(error),
                }
            },
            StatementKind::While(condition, block) => {
                while condition.eval(ctx)?.value.truthy() {
                    if ctx.shell.interrupt.load(Ordering::SeqCst) {
                        return Err(ShellErrorKind::Interrupt);
                    }

                    match block.eval(ctx, None) {
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

                        let mut variables: HashMap<Rc<str>, (bool, Value)> = HashMap::new();
                        variables.insert(name.into(), (false, item.to_owned()));
                        match block.eval(ctx, Some(variables)) {
                            Ok(()) => (),
                            Err(ShellErrorKind::Break) => break,
                            Err(ShellErrorKind::Continue) => continue,
                            Err(error) => return Err(error),
                        }
                    }
                    Ok(())
                }

                match value.value {
                    Value::List(list) => for_loop(ctx, list.iter().cloned(), &name, block),
                    Value::String(string) => for_loop(
                        ctx,
                        string.chars().map(|c| Value::from(String::from(c))),
                        &name,
                        block,
                    ),
                    Value::Range(range) => for_loop(
                        ctx,
                        Rc::unwrap_or_clone(range).map(Value::Int),
                        &name,
                        block,
                    ),
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
                    _ => Err(ShellErrorKind::InvalidIterator(value.value.to_type())),
                }
            }
            StatementKind::Fn(name, func) => {
                ctx.frame
                    .add_function(name.clone(), Rc::new((func.clone(), ctx.frame.clone())));
                Ok(())
            }
            StatementKind::TryCatch(block, catch) => {
                if let Err(e) = block.eval(ctx, None) {
                    if e.is_error() {
                        catch.eval(ctx, None)?;
                    } else {
                        return Err(e);
                    }
                }
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
            StatementKind::Block(block) => block.eval(ctx, None),
        }
    }
}
