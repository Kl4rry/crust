use std::{
    collections::{HashMap, VecDeque},
    io, mem,
    rc::Rc,
    thread,
};

use subprocess::{CommunicateError, Exec, ExitStatus, Pipeline, PopenError, Redirection};

use crate::{
    parser::{
        ast::{Literal, Variable},
        lexer::token::span::{Span, Spanned},
        shell_error::ShellErrorKind,
    },
    shell::{
        builtins::{self, functions::BulitinFn},
        frame::Frame,
        stream::{OutputStream, ValueStream},
        value::{SpannedValue, Type, Value},
    },
    P,
};

pub mod binop;
use binop::BinOp;

pub mod unop;
use unop::UnOp;

pub mod command;
use command::CommandPart;

pub mod argument;
use argument::Argument;

pub mod closure;
use closure::Closure;

use self::{binop::BinOpKind, unop::UnOpKind};
use super::{context::Context, statement::function::Function};

// used to implement comparison operators without duplciating code
macro_rules! compare_impl {
    ($arg_lhs:expr, $arg_rhs:expr, $arg_binop:expr, $op:tt) => {{
        #[inline(always)]
        fn compare_impl_fn(lhs: SpannedValue, rhs: SpannedValue, binop: BinOp) -> Result<SpannedValue, ShellErrorKind> {
            let (lhs, lhs_span) = lhs.into();
            let (rhs, rhs_span) = rhs.into();
            let span = lhs_span + rhs_span;

            match &lhs {
                Value::Int(number) => match &rhs {
                    Value::Int(rhs) => Ok(Value::Bool(number $op rhs).spanned(span)),
                    Value::Float(rhs) => Ok(Value::Bool((*number as f64) $op *rhs).spanned(span)),
                    Value::Bool(rhs) => Ok(Value::Bool(*number $op *rhs as i64).spanned(span)),
                    _ => Err(ShellErrorKind::InvalidBinaryOperand(
                        binop,
                        lhs.to_type(),
                        rhs.to_type(),
                        lhs_span,
                        rhs_span,
                    )),
                },
                Value::Float(number) => match &rhs {
                    Value::Int(rhs) => Ok(Value::Bool(*number $op *rhs as f64).spanned(span)),
                    Value::Float(rhs) => Ok(Value::Bool(number $op rhs).spanned(span)),
                    Value::Bool(rhs) => Ok(Value::Bool(*number $op *rhs as u8 as f64).spanned(span)),
                    _ => Err(ShellErrorKind::InvalidBinaryOperand(
                        binop,
                        lhs.to_type(),
                        rhs.to_type(),
                        lhs_span,
                        rhs_span,
                    )),
                },
                Value::Bool(boolean) => match &rhs {
                    Value::Int(rhs) => Ok(Value::Bool((*boolean as i64) $op *rhs).spanned(span)),
                    Value::Float(rhs) => Ok(Value::Bool((*boolean as u8 as f64) $op *rhs).spanned(span)),
                    Value::Bool(rhs) => Ok(Value::Bool(*boolean $op *rhs).spanned(span)),
                    _ => Err(ShellErrorKind::InvalidBinaryOperand(
                        binop,
                        lhs.to_type(),
                        rhs.to_type(),
                        lhs_span,
                        rhs_span,
                    )),
                },
                Value::String(string) => match &rhs {
                    Value::String(rhs) => Ok(Value::Bool(string $op rhs).spanned(span)),
                    _ => Err(ShellErrorKind::InvalidBinaryOperand(
                        binop,
                        lhs.to_type(),
                        rhs.to_type(),
                        lhs_span,
                        rhs_span,
                    )),
                },
                _ => Err(ShellErrorKind::InvalidBinaryOperand(
                    binop,
                        lhs.to_type(),
                        rhs.to_type(),
                        lhs_span,
                        rhs_span,
                )),
            }
        }
        compare_impl_fn($arg_lhs, $arg_rhs, $arg_binop)
    }};
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Call(Vec<CommandPart>, Vec<Argument>),
    Pipe(Vec<Expr>),
    Variable(Variable),
    Binary(BinOp, P<Expr>, P<Expr>),
    Unary(UnOp, P<Expr>),
    Literal(Literal),
    SubExpr(P<Expr>),
    Column(P<Expr>, String),
    ErrorCheck(P<Expr>),
    Index { expr: P<Expr>, index: P<Expr> },
    Closure(Rc<Closure>),
}

impl ExprKind {
    pub fn spanned(self, span: Span) -> Expr {
        Expr { kind: self, span }
    }
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

impl Expr {
    #[inline(always)]
    pub fn wrap(self, unop: Option<UnOp>) -> Self {
        match unop {
            Some(unop) => ExprKind::Unary(unop, P::new(self)).spanned(unop.span),
            None => self,
        }
    }

    pub fn eval(&self, ctx: &mut Context) -> Result<SpannedValue, ShellErrorKind> {
        match &self.kind {
            ExprKind::Call(_, _) => {
                unreachable!("calls must always be in a pipeline, bare calls are a bug")
            }
            ExprKind::Closure(closure) => Ok(Value::Closure(Rc::new((
                closure.clone(),
                ctx.frame.clone(),
            )))
            .spanned(closure.span)),
            ExprKind::ErrorCheck(expr) => match expr.eval(ctx) {
                Ok(_) => Ok(Value::Bool(true).spanned(expr.span)),
                Err(err) => {
                    if err.is_error() {
                        let code = err.exit_status();
                        Ok(Value::Bool(code == 0).spanned(expr.span))
                    } else {
                        Err(err)
                    }
                }
            },
            ExprKind::Column(expr, col) => {
                let (value, span) = expr.eval(ctx)?.into();
                match value {
                    Value::Map(map) => match map.get(col.as_str()) {
                        Some(value) => Ok(value.clone().spanned(span)),
                        None => Err(ShellErrorKind::ColumnNotFound(col.to_string())),
                    },
                    Value::Table(table) => Ok(Value::from(table.column(col)?).spanned(span)),
                    _ => Err(ShellErrorKind::NoColumns(value.to_type())),
                }
            }
            ExprKind::Index { expr, index } => {
                let (value, span) = expr.eval(ctx)?.into();
                let index = index.eval(ctx)?;
                let total_span = span + index.span;
                // TODO use cow here and just clone once
                match value {
                    Value::List(list) => Ok(list
                        .get(index.try_as_index(list.len())?)
                        .unwrap()
                        .clone()
                        .spanned(total_span)),
                    Value::Table(table) => Ok(Value::from(table.row(index)?).spanned(total_span)),
                    Value::String(string) => {
                        let chars: Vec<char> = string.chars().collect();
                        let c = chars[index.try_as_index(chars.len())?];
                        Ok(Value::from(String::from(c)).spanned(total_span))
                    }
                    _ => Err(ShellErrorKind::NotIndexable(value.to_type(), span)),
                }
            }
            ExprKind::Literal(literal) => literal.eval(ctx),
            ExprKind::Variable(variable) => variable.eval(ctx),
            ExprKind::Unary(unop, expr) => {
                let (value, span) = expr.eval(ctx)?.into();
                let span = unop.span + span;
                match unop.kind {
                    UnOpKind::Neg => match &value {
                        Value::Int(int) => Ok(Value::Int(-*int).spanned(span)),
                        Value::Float(float) => Ok(Value::Float(-*float).spanned(span)),
                        Value::Bool(boolean) => Ok(Value::Int(-(*boolean as i64)).spanned(span)),
                        _ => Err(ShellErrorKind::InvalidUnaryOperand(
                            *unop,
                            value.to_type(),
                            unop.span,
                        )),
                    },
                    UnOpKind::Not => Ok(Value::Bool(!value.truthy()).spanned(span)),
                }
            }
            ExprKind::Binary(binop, lhs, rhs) => match binop.kind {
                BinOpKind::Match => {
                    let lhs = lhs.eval(ctx)?;
                    let rhs = rhs.eval(ctx)?;
                    let span = lhs.span + rhs.span;
                    Ok(Value::Bool(lhs.try_match(rhs, binop.span)?).spanned(span))
                }
                BinOpKind::NotMatch => {
                    let lhs = lhs.eval(ctx)?;
                    let rhs = rhs.eval(ctx)?;
                    let span = lhs.span + rhs.span;
                    Ok(Value::Bool(!lhs.try_match(rhs, binop.span)?).spanned(span))
                }
                BinOpKind::Range => {
                    let lhs_value = lhs.eval(ctx)?;
                    let rhs_value = rhs.eval(ctx)?;

                    let lhs_span = lhs_value.span;
                    let rhs_span = rhs_value.span;

                    let lhs = lhs_value.value.try_as_int();
                    let rhs = rhs_value.value.try_as_int();

                    if lhs.is_none() || rhs.is_none() {
                        return Err(ShellErrorKind::InvalidBinaryOperand(
                            BinOpKind::Range.spanned(binop.span),
                            lhs_value.value.to_type(),
                            rhs_value.value.to_type(),
                            lhs_span,
                            rhs_span,
                        ));
                    }

                    // SAFETY
                    // this is safe because we check neither lhs or rhs is none
                    unsafe {
                        let lhs = lhs.unwrap_unchecked();
                        let rhs = rhs.unwrap_unchecked();
                        Ok(Value::from(lhs..rhs).spanned(lhs_span + rhs_span))
                    }
                }
                BinOpKind::Add => {
                    let lhs = lhs.eval(ctx)?;
                    let rhs = rhs.eval(ctx)?;
                    lhs.try_add(rhs, binop.span)
                }
                BinOpKind::Sub => {
                    let lhs = lhs.eval(ctx)?;
                    let rhs = rhs.eval(ctx)?;
                    lhs.try_sub(rhs, binop.span)
                }
                BinOpKind::Mul => {
                    let lhs = lhs.eval(ctx)?;
                    let rhs = rhs.eval(ctx)?;
                    lhs.try_mul(rhs, binop.span)
                }
                BinOpKind::Div => {
                    let lhs = lhs.eval(ctx)?;
                    let rhs = rhs.eval(ctx)?;
                    lhs.try_div(rhs, binop.span)
                }
                BinOpKind::Expo => {
                    let lhs = lhs.eval(ctx)?;
                    let rhs = rhs.eval(ctx)?;
                    lhs.try_expo(rhs, binop.span)
                }
                BinOpKind::Mod => {
                    let lhs = lhs.eval(ctx)?;
                    let rhs = rhs.eval(ctx)?;
                    lhs.try_mod(rhs, binop.span)
                }
                // The == operator (equality)
                BinOpKind::Eq => {
                    let lhs = lhs.eval(ctx)?;
                    let rhs = rhs.eval(ctx)?;
                    Ok(Value::Bool(lhs.value == rhs.value).spanned(binop.span))
                }
                // The != operator (not equal to)
                BinOpKind::Ne => {
                    let lhs = lhs.eval(ctx)?;
                    let rhs = rhs.eval(ctx)?;
                    Ok(Value::Bool(lhs.value != rhs.value).spanned(binop.span))
                }

                // all the ordering operators are the same except for the operator
                // the this is why the macro is used

                // The < operator (less than)
                BinOpKind::Lt => {
                    let lhs = lhs.eval(ctx)?;
                    let rhs = rhs.eval(ctx)?;
                    compare_impl!(lhs, rhs, *binop, <)
                }
                // The <= operator (less than or equal to)
                BinOpKind::Le => {
                    let lhs = lhs.eval(ctx)?;
                    let rhs = rhs.eval(ctx)?;
                    compare_impl!(lhs, rhs, *binop, <=)
                }
                // The >= operator (greater than or equal to)
                BinOpKind::Ge => {
                    let lhs = lhs.eval(ctx)?;
                    let rhs = rhs.eval(ctx)?;
                    compare_impl!(lhs, rhs, *binop, >=)
                }
                // The > operator (greater than)
                BinOpKind::Gt => {
                    let lhs = lhs.eval(ctx)?;
                    let rhs = rhs.eval(ctx)?;
                    compare_impl!(lhs, rhs, *binop, >)
                }
                BinOpKind::And => {
                    let (lhs, lhs_span) = lhs.eval(ctx)?.into();
                    let (rhs, rhs_span) = rhs.eval(ctx)?.into();
                    let span = lhs_span + rhs_span;
                    Ok(Value::Bool(lhs.truthy() && rhs.truthy()).spanned(span))
                }
                BinOpKind::Or => {
                    let (lhs, lhs_span) = lhs.eval(ctx)?.into();
                    let (rhs, rhs_span) = rhs.eval(ctx)?.into();
                    let span = lhs_span + rhs_span;
                    Ok(Value::Bool(lhs.truthy() || rhs.truthy()).spanned(span))
                }
            },
            ExprKind::Pipe(calls) => {
                let pipe_span = calls.first().unwrap().span + calls.last().unwrap().span;
                let mut calls = calls.iter().peekable();
                let mut capture_output = Spanned::new(OutputStream::new_capture(), Span::new(0, 0));
                // If the first thing in the pipeline is not a command we eval it first
                if !matches!(calls.peek().unwrap().kind, ExprKind::Call(..)) {
                    let first = calls.next().unwrap();
                    capture_output.span = first.span;
                    capture_output.inner.push(first.eval(ctx)?.into());
                }

                let mut expanded_calls = VecDeque::new();
                for callable in calls {
                    match &callable.kind {
                        ExprKind::Call(cmd, args) => {
                            let (cmd, args) = expand_call(ctx, cmd, args)?;
                            expanded_calls.push_back(get_call_type(ctx, cmd, args)?);
                        }
                        _ => unreachable!(),
                    }
                }

                let mut execs: Vec<(Exec, String, Span)> = Vec::new();

                while let Some(call_type) = expanded_calls.pop_front() {
                    match call_type {
                        CallType::External(exec, name, span) => {
                            execs.push((*exec, name, span));
                        }
                        CallType::Builtin(builtin, args, span) => {
                            let mut stream = if execs.is_empty() {
                                let mut stream = OutputStream::new_capture();
                                mem::swap(&mut capture_output.inner, &mut stream);
                                stream.into_value_stream()
                            } else {
                                let value = run_pipeline(
                                    ctx,
                                    execs,
                                    true,
                                    Spanned::new(
                                        capture_output.inner.into_value_stream(),
                                        capture_output.span,
                                    ),
                                )?
                                .unwrap();
                                capture_output.inner = OutputStream::new_capture();
                                capture_output.span = span;
                                execs = Vec::new();
                                ValueStream::from_value(value)
                            };

                            let temp_output_cap = if expanded_calls.is_empty() {
                                &mut *ctx.output
                            } else {
                                capture_output.inner = OutputStream::new_capture();
                                capture_output.span = span;
                                &mut capture_output.inner
                            };
                            let mut ctx = Context {
                                shell: ctx.shell,
                                frame: ctx.frame.clone(),
                                output: temp_output_cap,
                                input: &mut stream,
                                src: ctx.src.clone(),
                            };
                            builtin(&mut ctx, args)?;
                            ctx.shell.set_status(ExitStatus::Exited(0));
                        }
                        CallType::Internal(func, args, span) => {
                            let mut stream = if execs.is_empty() {
                                let mut stream = OutputStream::new_capture();
                                mem::swap(&mut capture_output.inner, &mut stream);
                                stream.into_value_stream()
                            } else {
                                let value = run_pipeline(
                                    ctx,
                                    execs,
                                    true,
                                    Spanned::new(
                                        capture_output.inner.into_value_stream(),
                                        capture_output.span,
                                    ),
                                )?
                                .unwrap();
                                capture_output.inner = OutputStream::new_capture();
                                capture_output.span = span;
                                execs = Vec::new();
                                ValueStream::from_value(value)
                            };

                            let (function, frame) = &*func;

                            let Function {
                                parameters,
                                block,
                                name,
                                src,
                                arg_span,
                                ..
                            } = &**function;

                            if parameters.len() != args.len() {
                                return Err(ShellErrorKind::IncorrectArgumentCount {
                                    name: Some(name.clone()),
                                    expected: parameters.len(),
                                    recived: args.len(),
                                    arg_span: *arg_span,
                                    src: src.clone(),
                                });
                            }

                            let mut input_vars = HashMap::new();
                            for (var, arg) in parameters.iter().zip(args.iter()) {
                                input_vars.insert(var.name.clone(), (false, arg.clone().value));
                            }

                            let temp_output_cap = if expanded_calls.is_empty() {
                                &mut *ctx.output
                            } else {
                                capture_output.inner = OutputStream::new_capture();
                                capture_output.span = span;
                                &mut capture_output.inner
                            };
                            let ctx = &mut Context {
                                shell: ctx.shell,
                                frame: frame.clone(),
                                output: temp_output_cap,
                                input: &mut stream,
                                src: ctx.src.clone(),
                            };
                            block.eval(ctx, Some(input_vars))?;
                            ctx.shell.set_status(ExitStatus::Exited(0));
                        }
                    }
                }

                if !execs.is_empty() {
                    if ctx.output.is_capture() {
                        let value = run_pipeline(
                            ctx,
                            execs,
                            true,
                            Spanned::new(
                                capture_output.inner.into_value_stream(),
                                capture_output.span,
                            ),
                        )?
                        .unwrap();
                        ctx.output.push(value);
                    } else {
                        run_pipeline(
                            ctx,
                            execs,
                            false,
                            Spanned::new(
                                capture_output.inner.into_value_stream(),
                                capture_output.span,
                            ),
                        )?;
                    }
                }

                Ok(Value::Null.spanned(pipe_span))
            }
            ExprKind::SubExpr(expr) => {
                if matches!(expr.kind, ExprKind::Call { .. } | ExprKind::Pipe { .. }) {
                    let mut capture = OutputStream::new_capture();
                    let ctx = &mut Context {
                        shell: ctx.shell,
                        frame: ctx.frame.clone(),
                        output: &mut capture,
                        input: &mut ValueStream::new(),
                        src: ctx.src.clone(),
                    };
                    let span = expr.span;
                    expr.eval(ctx)?;
                    Ok(capture.into_value_stream().unpack().spanned(span))
                } else {
                    expr.eval(ctx)
                }
            }
        }
    }
}

pub fn try_bytes_to_value(bytes: Vec<u8>) -> Value {
    String::from_utf8(bytes)
        .map(Value::from)
        .unwrap_or_else(|e| Value::from(e.into_bytes()))
}

fn run_pipeline(
    ctx: &mut Context,
    mut execs: Vec<(Exec, String, Span)>,
    capture_output: bool,
    input: Spanned<ValueStream>,
) -> Result<Option<Value>, ShellErrorKind> {
    let Spanned {
        inner: input,
        span: input_span,
    } = input;

    let mut input_data = Vec::new();
    for value in input.into_iter() {
        let spanned: SpannedValue = value.spanned(input_span);
        match spanned.value {
            Value::String(..)
            | Value::Bool(..)
            | Value::Int(..)
            | Value::Float(..)
            | Value::Range(..)
            | Value::List(..) => {
                let mut strings = Vec::new();
                spanned.try_expand_to_strings(&mut strings)?;
                for string in strings {
                    input_data.extend_from_slice(string.as_bytes());
                    input_data.push(b'\n');
                }
            }
            Value::Binary(data) => {
                // TODO figure out if there should be a newline here
                input_data.extend_from_slice(&data);
            }
            _ => {
                return Err(ShellErrorKind::InvalidPipelineInput {
                    expected: Type::STRING,
                    recived: spanned.value.to_type(),
                });
            }
        }
    }

    let input_data: Option<Vec<u8>> = if input_data.is_empty() {
        None
    } else {
        Some(input_data)
    };

    let stdin = if input_data.is_some() {
        Redirection::Pipe
    } else {
        Redirection::None
    };

    let env = ctx.frame.env();
    if execs.len() == 1 {
        let (exec, name) = if capture_output {
            let (exec, name, _) = execs.pop().unwrap();
            (exec.stdout(Redirection::Pipe), name)
        } else {
            execs.pop().map(|(exec, name, _)| (exec, name)).unwrap()
        };
        let exec = exec.stdin(stdin).env_clear().env_extend(&env);

        let mut child = exec.popen().map_err(|e| popen_to_shell_err(e, name))?;
        ctx.shell.set_child(child.pid());
        let res = if capture_output {
            let mut com = child.communicate_start(input_data);
            let t = thread::spawn::<_, Result<Option<Vec<u8>>, CommunicateError>>(move || {
                let (out, _) = com.read()?;
                Ok(out)
            });

            let status = child.wait()?;
            let output = t.join().unwrap()?.map(try_bytes_to_value);
            if !status.success() {
                return Err(ShellErrorKind::ExternalExitCode(status));
            }
            ctx.shell.set_status(status);
            Ok(output)
        } else {
            let _ = child.communicate_start(input_data);
            let status = child.wait()?;
            if !status.success() {
                return Err(ShellErrorKind::ExternalExitCode(status));
            }
            ctx.shell.set_status(status);
            Ok(None)
        };
        ctx.shell.set_child(None);
        res
    } else {
        // TODO this also need to be turned into a command not found error
        let execs = execs
            .into_iter()
            .map(|(exec, _, _)| exec.env_clear().env_extend(&env));
        let pipeline = Pipeline::from_exec_iter(execs)
            .stdin(stdin)
            .stdout(if capture_output {
                Redirection::Pipe
            } else {
                Redirection::None
            });
        let mut children: Vec<subprocess::Popen> = pipeline.popen()?;
        children
            .first_mut()
            .unwrap()
            .communicate_bytes(input_data.as_deref())
            .map_err(|err| ShellErrorKind::Io(None, err))?;
        ctx.shell.set_child(children.last_mut().unwrap().pid());

        if capture_output {
            let mut com = children.last_mut().unwrap().communicate_start(None);
            let t = thread::spawn::<_, Result<Option<Vec<u8>>, CommunicateError>>(move || {
                let (out, _) = com.read()?;
                Ok(out)
            });
            let status = children.last_mut().unwrap().wait()?;
            ctx.shell.set_child(None);
            if !status.success() {
                return Err(ShellErrorKind::ExternalExitCode(status));
            }
            ctx.shell.set_status(status);
            Ok(t.join().unwrap()?.map(try_bytes_to_value))
        } else {
            let status = children.last_mut().unwrap().wait()?;
            ctx.shell.set_child(None);
            if !status.success() {
                return Err(ShellErrorKind::ExternalExitCode(status));
            }
            ctx.shell.set_status(status);
            Ok(None)
        }
    }
}

fn get_call_type(
    ctx: &mut Context,
    cmd: Spanned<String>,
    args: Vec<SpannedValue>,
) -> Result<CallType, ShellErrorKind> {
    let span = cmd.span;
    if let Some(builtin) = builtins::functions::get_builtin(&cmd.inner) {
        return Ok(CallType::Builtin(builtin, args, span));
    }

    for frame in ctx.frame.clone() {
        if let Some(func) = frame.get_function(&cmd.inner) {
            return Ok(CallType::Internal(func, args, span));
        }
    }

    let cmd = match ctx.shell.find_exe(&cmd.inner) {
        Some(cmd) => cmd,
        None => cmd.inner,
    };

    let mut str_args = Vec::new();
    for arg in args {
        arg.try_expand_to_strings(&mut str_args)?;
    }

    Ok(CallType::External(
        P::new(Exec::cmd(cmd.clone()).args(&str_args)),
        cmd,
        span,
    ))
}

fn expand_call(
    ctx: &mut Context,
    commandparts: &[CommandPart],
    args: &[Argument],
) -> Result<(Spanned<String>, Vec<SpannedValue>), ShellErrorKind> {
    let mut expanded_args = Vec::new();
    for arg in args {
        expanded_args.push(arg.eval(ctx)?);
    }

    let cmd_span = commandparts.first().unwrap().span + commandparts.last().unwrap().span;
    let mut command = String::new();
    for part in commandparts.iter() {
        command.push_str(&part.eval(ctx)?);
    }

    if let Some(alias) = ctx.shell.aliases.get(&command) {
        let mut split = alias.split_whitespace();
        command = split.next().unwrap().to_string();
        let mut args: Vec<_> = split
            .map(|s| Value::from(s.to_string()).spanned(cmd_span))
            .collect();
        args.extend(expanded_args);
        expanded_args = args;
    }
    Ok((Spanned::new(command, cmd_span), expanded_args))
}

pub enum CallType {
    Builtin(BulitinFn, Vec<SpannedValue>, Span),
    Internal(Rc<(Rc<Function>, Frame)>, Vec<SpannedValue>, Span),
    External(P<Exec>, String, Span),
}

fn popen_to_shell_err(error: PopenError, name: String) -> ShellErrorKind {
    match error {
        PopenError::IoError(err) => match err.kind() {
            io::ErrorKind::NotFound => ShellErrorKind::CommandNotFound(name),
            io::ErrorKind::PermissionDenied => ShellErrorKind::CommandPermissionDenied(name),
            _ => ShellErrorKind::Io(None, err),
        },
        error => ShellErrorKind::Popen(error),
    }
}
