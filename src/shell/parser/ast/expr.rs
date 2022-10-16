use std::{
    collections::{HashMap, VecDeque},
    io, mem,
    rc::Rc,
    thread,
};

use subprocess::{CommunicateError, Exec, Pipeline, PopenError, Redirection};

use crate::{
    parser::{
        ast::{Literal, Variable},
        shell_error::ShellErrorKind,
    },
    shell::{
        builtins::{self, functions::BulitinFn},
        frame::Frame,
        stream::{OutputStream, ValueStream},
        value::{Type, Value},
        Shell,
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

use super::Block;

// used to implement comparison operators without duplciating code
macro_rules! compare_impl {
    ($arg_lhs:expr, $arg_rhs:expr, $arg_binop:expr, $op:tt) => {{
        #[inline(always)]
        fn compare_impl_fn(lhs: Value, rhs: Value, binop: BinOp) -> Result<Value, ShellErrorKind> {
            match &lhs {
                Value::Int(number) => match &rhs {
                    Value::Int(rhs) => Ok(Value::Bool(number $op rhs)),
                    Value::Float(rhs) => Ok(Value::Bool((*number as f64) $op *rhs)),
                    Value::Bool(rhs) => Ok(Value::Bool(*number $op *rhs as i64)),
                    _ => Err(ShellErrorKind::InvalidBinaryOperand(
                        binop,
                        lhs.to_type(),
                        rhs.to_type(),
                    )),
                },
                Value::Float(number) => match &rhs {
                    Value::Int(rhs) => Ok(Value::Bool(*number $op *rhs as f64)),
                    Value::Float(rhs) => Ok(Value::Bool(number $op rhs)),
                    Value::Bool(rhs) => Ok(Value::Bool(*number $op *rhs as u8 as f64)),
                    _ => Err(ShellErrorKind::InvalidBinaryOperand(
                        binop,
                        lhs.to_type(),
                        rhs.to_type(),
                    )),
                },
                Value::Bool(boolean) => match &rhs {
                    Value::Int(rhs) => Ok(Value::Bool((*boolean as i64) $op *rhs)),
                    Value::Float(rhs) => Ok(Value::Bool((*boolean as u8 as f64) $op *rhs)),
                    Value::Bool(rhs) => Ok(Value::Bool(*boolean $op *rhs)),
                    _ => Err(ShellErrorKind::InvalidBinaryOperand(
                        binop,
                        lhs.to_type(),
                        rhs.to_type(),
                    )),
                },
                Value::String(string) => match &rhs {
                    Value::String(rhs) => Ok(Value::Bool(string $op rhs)),
                    _ => Err(ShellErrorKind::InvalidBinaryOperand(
                        binop,
                        lhs.to_type(),
                        rhs.to_type(),
                    )),
                },
                _ => Err(ShellErrorKind::InvalidBinaryOperand(
                    binop,
                    lhs.to_type(),
                    rhs.to_type(),
                )),
            }
        }
        compare_impl_fn($arg_lhs, $arg_rhs, $arg_binop)
    }};
}

#[derive(Debug, Clone)]
pub enum Expr {
    Call(Vec<CommandPart>, Vec<Argument>),
    Pipe(Vec<Expr>),
    Variable(Variable),
    Binary(BinOp, P<Expr>, P<Expr>),
    Unary(UnOp, P<Expr>),
    Literal(Literal),
    SubExpr(P<Expr>),
    Column(P<Expr>, String),
    Index { expr: P<Expr>, index: P<Expr> },
}

impl Expr {
    #[inline(always)]
    pub fn wrap(self, unop: Option<UnOp>) -> Self {
        match unop {
            Some(unop) => Expr::Unary(unop, P::new(self)),
            None => self,
        }
    }

    pub fn eval(
        &self,
        shell: &mut Shell,
        frame: &mut Frame,
        output: &mut OutputStream,
    ) -> Result<Value, ShellErrorKind> {
        match self {
            Self::Call(_, _) => {
                unreachable!("calls must always be in a pipeline, bare calls are not allowed")
            }
            Self::Column(expr, col) => {
                let value = expr.eval(shell, frame, output)?;
                match value {
                    Value::Map(map) => match map.get(col) {
                        Some(value) => Ok(value.clone()),
                        None => Err(ShellErrorKind::ColumnNotFound(col.to_string())),
                    },
                    Value::Table(table) => Ok(Value::from(table.column(col)?)),
                    _ => Err(ShellErrorKind::NoColumns(value.to_type())),
                }
            }
            Self::Index { expr, index } => {
                let value = expr.eval(shell, frame, output)?;
                let index = index.eval(shell, frame, output)?;
                // TODO use cow here and just clone once
                match value {
                    Value::List(list) => {
                        Ok(list.get(index.try_as_index(list.len())?).unwrap().clone())
                    }
                    Value::Table(table) => {
                        Ok(Value::from(table.row(index.try_as_index(table.len())?)?))
                    }
                    Value::String(string) => {
                        let chars: Vec<char> = string.chars().collect();
                        let c = chars[index.try_as_index(chars.len())?];
                        Ok(Value::from(String::from(c)))
                    }
                    _ => Err(ShellErrorKind::NotIndexable(value.to_type())),
                }
            }
            Self::Literal(literal) => literal.eval(shell, frame, output),
            Self::Variable(variable) => variable.eval(shell, frame),
            Self::Unary(unop, expr) => {
                let value = expr.eval(shell, frame, output)?;
                match unop {
                    UnOp::Neg => match &value {
                        Value::Int(int) => Ok(Value::Int(-*int)),
                        Value::Float(float) => Ok(Value::Float(-*float)),
                        Value::Bool(boolean) => Ok(Value::Int(-(*boolean as i64))),
                        _ => Err(ShellErrorKind::InvalidUnaryOperand(*unop, value.to_type())),
                    },
                    UnOp::Not => Ok(Value::Bool(!value.truthy())),
                }
            }
            Self::Binary(binop, lhs, rhs) => match binop {
                BinOp::Match => {
                    let lhs = lhs.eval(shell, frame, output)?;
                    let rhs = rhs.eval(shell, frame, output)?;
                    Ok(Value::Bool(lhs.try_match(rhs)?))
                }
                BinOp::NotMatch => {
                    let lhs = lhs.eval(shell, frame, output)?;
                    let rhs = rhs.eval(shell, frame, output)?;
                    Ok(Value::Bool(!lhs.try_match(rhs)?))
                }
                BinOp::Range => {
                    let lhs_value = lhs.eval(shell, frame, output)?;
                    let rhs_value = rhs.eval(shell, frame, output)?;

                    let lhs = lhs_value.try_as_int();
                    let rhs = rhs_value.try_as_int();

                    if lhs.is_none() || rhs.is_none() {
                        return Err(ShellErrorKind::InvalidBinaryOperand(
                            BinOp::Range,
                            lhs_value.to_type(),
                            rhs_value.to_type(),
                        ));
                    }

                    // SAFETY
                    // this is safe because we check neither lhs or rhs is none
                    unsafe {
                        let lhs = lhs.unwrap_unchecked();
                        let rhs = rhs.unwrap_unchecked();
                        Ok(Value::from(lhs..rhs))
                    }
                }
                BinOp::Add => {
                    let lhs = lhs.eval(shell, frame, output)?;
                    let rhs = rhs.eval(shell, frame, output)?;
                    lhs.try_add(rhs)
                }
                BinOp::Sub => {
                    let lhs = lhs.eval(shell, frame, output)?;
                    let rhs = rhs.eval(shell, frame, output)?;
                    lhs.try_sub(rhs)
                }
                BinOp::Mul => {
                    let lhs = lhs.eval(shell, frame, output)?;
                    let rhs = rhs.eval(shell, frame, output)?;
                    lhs.try_mul(rhs)
                }
                BinOp::Div => {
                    let lhs = lhs.eval(shell, frame, output)?;
                    let rhs = rhs.eval(shell, frame, output)?;
                    lhs.try_div(rhs)
                }
                BinOp::Expo => {
                    let lhs = lhs.eval(shell, frame, output)?;
                    let rhs = rhs.eval(shell, frame, output)?;
                    lhs.try_expo(rhs)
                }
                BinOp::Mod => {
                    let lhs = lhs.eval(shell, frame, output)?;
                    let rhs = rhs.eval(shell, frame, output)?;
                    lhs.try_mod(rhs)
                }
                // The == operator (equality)
                BinOp::Eq => {
                    let lhs = lhs.eval(shell, frame, output)?;
                    let rhs = rhs.eval(shell, frame, output)?;
                    Ok(Value::Bool(lhs == rhs))
                }
                // The != operator (not equal to)
                BinOp::Ne => {
                    let lhs = lhs.eval(shell, frame, output)?;
                    let rhs = rhs.eval(shell, frame, output)?;
                    Ok(Value::Bool(lhs != rhs))
                }

                // all the ordering operators are the same except for the operator
                // the this is why the macro is used

                // The < operator (less than)
                BinOp::Lt => {
                    let lhs = lhs.eval(shell, frame, output)?;
                    let rhs = rhs.eval(shell, frame, output)?;
                    compare_impl!(lhs, rhs, *binop, <)
                }
                // The <= operator (less than or equal to)
                BinOp::Le => {
                    let lhs = lhs.eval(shell, frame, output)?;
                    let rhs = rhs.eval(shell, frame, output)?;
                    compare_impl!(lhs, rhs, *binop, <=)
                }
                // The >= operator (greater than or equal to)
                BinOp::Ge => {
                    let lhs = lhs.eval(shell, frame, output)?;
                    let rhs = rhs.eval(shell, frame, output)?;
                    compare_impl!(lhs, rhs, *binop, >=)
                }
                // The > operator (greater than)
                BinOp::Gt => {
                    let lhs = lhs.eval(shell, frame, output)?;
                    let rhs = rhs.eval(shell, frame, output)?;
                    compare_impl!(lhs, rhs, *binop, >)
                }
                BinOp::And => Ok(Value::Bool(
                    lhs.eval(shell, frame, output)?.truthy()
                        && rhs.eval(shell, frame, output)?.truthy(),
                )),
                BinOp::Or => Ok(Value::Bool(
                    lhs.eval(shell, frame, output)?.truthy()
                        || rhs.eval(shell, frame, output)?.truthy(),
                )),
            },
            Self::Pipe(calls) => {
                let mut calls = calls.iter().peekable();
                let mut capture_output = OutputStream::new_capture();
                if !matches!(calls.peek().unwrap(), Expr::Call(..)) {
                    capture_output.push(calls.next().unwrap().eval(shell, frame, output)?);
                }

                let mut expanded_calls = VecDeque::new();
                for callable in calls {
                    match callable {
                        Self::Call(cmd, args) => {
                            let (cmd, args) = expand_call(shell, frame, cmd, args, output)?;
                            expanded_calls.push_back(get_call_type(shell, frame, cmd, args));
                        }
                        _ => unreachable!(),
                    }
                }

                let mut execs: Vec<(Exec, String)> = Vec::new();

                while let Some(call_type) = expanded_calls.pop_front() {
                    match call_type {
                        CallType::External(exec, name) => {
                            execs.push((*exec, name));
                        }
                        CallType::Builtin(builtin, args) => {
                            let stream = if execs.is_empty() {
                                let mut stream = OutputStream::new_capture();
                                mem::swap(&mut capture_output, &mut stream);
                                stream.into_value_stream()
                            } else {
                                let value = Value::from(
                                    run_pipeline(
                                        shell,
                                        frame,
                                        execs,
                                        true,
                                        capture_output.into_value_stream(),
                                    )?
                                    .unwrap()
                                    .to_string(),
                                );
                                capture_output = OutputStream::new_capture();
                                execs = Vec::new();
                                ValueStream::from_value(value)
                            };

                            let temp_output_cap = if expanded_calls.is_empty() {
                                &mut *output
                            } else {
                                capture_output = OutputStream::new_capture();
                                &mut capture_output
                            };
                            builtin(shell, frame, args, stream, temp_output_cap)?;
                        }
                        CallType::Internal(func, args) => {
                            let stream = if execs.is_empty() {
                                let mut stream = OutputStream::new_capture();
                                mem::swap(&mut capture_output, &mut stream);
                                stream.into_value_stream()
                            } else {
                                let value = Value::from(
                                    run_pipeline(
                                        shell,
                                        frame,
                                        execs,
                                        true,
                                        capture_output.into_value_stream(),
                                    )?
                                    .unwrap()
                                    .to_string(),
                                );
                                capture_output = OutputStream::new_capture();
                                execs = Vec::new();
                                ValueStream::from_value(value)
                            };

                            let (vars, block) = &*func;
                            let mut input_vars = HashMap::new();
                            for (i, var) in vars.iter().enumerate() {
                                match args.get(i) {
                                    Some(arg) => {
                                        input_vars.insert(var.name.clone(), (false, arg.clone()));
                                    }
                                    None => {
                                        return Err(ShellErrorKind::ToFewArguments {
                                            // this should be function name
                                            name: String::from("function"),
                                            expected: vars.len(),
                                            recived: args.len(),
                                        });
                                    }
                                }
                            }

                            let temp_output_cap = if expanded_calls.is_empty() {
                                &mut *output
                            } else {
                                capture_output = OutputStream::new_capture();
                                &mut capture_output
                            };
                            block.eval(
                                shell,
                                frame.clone(),
                                Some(input_vars),
                                Some(stream),
                                temp_output_cap,
                            )?;
                        }
                    }
                }

                if !execs.is_empty() {
                    if output.is_capture() {
                        let value = Value::String(Rc::new(
                            run_pipeline(
                                shell,
                                frame,
                                execs,
                                true,
                                capture_output.into_value_stream(),
                            )?
                            .unwrap(),
                        ));
                        output.push(value);
                    } else {
                        run_pipeline(
                            shell,
                            frame,
                            execs,
                            false,
                            capture_output.into_value_stream(),
                        )?;
                    }
                }

                Ok(Value::Null)
            }
            Self::SubExpr(expr) => {
                if matches!(**expr, Self::Call { .. } | Self::Pipe { .. }) {
                    let mut capture = OutputStream::new_capture();
                    expr.eval(shell, frame, &mut capture)?;
                    Ok(capture.into_value_stream().unpack())
                } else {
                    expr.eval(shell, frame, output)
                }
            }
        }
    }
}

fn run_pipeline(
    shell: &mut Shell,
    frame: &mut Frame,
    mut execs: Vec<(Exec, String)>,
    capture_output: bool,
    input: ValueStream,
) -> Result<Option<String>, ShellErrorKind> {
    let mut input_string = String::new();
    for value in input {
        match value {
            Value::String(text) => {
                input_string.push_str(&text);
                input_string.push('\n');
            }
            _ => {
                return Err(ShellErrorKind::InvalidPipelineInput {
                    expected: Type::STRING,
                    recived: value.to_type(),
                })
            }
        }
    }
    let input_data: Option<Vec<u8>> = if input_string.is_empty() {
        None
    } else {
        Some(input_string.into())
    };

    let stdin = if input_data.is_some() {
        Redirection::Pipe
    } else {
        Redirection::None
    };

    let env = frame.env();
    if execs.len() == 1 {
        let (exec, name) = if capture_output {
            let (exec, name) = execs.pop().unwrap();
            (exec.stdout(Redirection::Pipe), name)
        } else {
            execs.pop().unwrap()
        };
        let exec = exec.stdin(stdin).env_clear().env_extend(&env);

        let mut child = exec.popen().map_err(|e| popen_to_shell_err(e, name))?;
        shell.set_child(child.pid());
        let res = if capture_output {
            let mut com = child.communicate_start(input_data);
            let t = thread::spawn::<_, Result<Option<String>, CommunicateError>>(move || {
                let (out, _) = com.read_string()?;
                Ok(out)
            });

            shell.set_status(child.wait()?);
            Ok(t.join().unwrap()?)
        } else {
            let _ = child.communicate_start(input_data);
            shell.set_status(child.wait()?);
            Ok(None)
        };
        shell.set_child(None);
        res
    } else {
        // TODO this also need to be turned into a command not found error
        let execs = execs
            .into_iter()
            .map(|(exec, _)| exec.env_clear().env_extend(&env));
        let pipeline = Pipeline::from_exec_iter(execs).stdin(stdin);
        let mut children = pipeline.popen()?;
        children
            .first_mut()
            .unwrap()
            .communicate_bytes(input_data.as_deref())
            .map_err(|err| ShellErrorKind::Io(None, err))?;
        let last = children.last_mut().unwrap();
        shell.set_child(last.pid());

        if capture_output {
            shell.set_child(None);
            Ok(None)
        } else {
            shell.set_status(last.wait()?);
            shell.set_child(None);
            Ok(None)
        }
    }
}

fn get_call_type(shell: &Shell, frame: &mut Frame, cmd: String, args: Vec<Value>) -> CallType {
    if let Some(builtin) = builtins::functions::get_builtin(&cmd) {
        return CallType::Builtin(builtin, args);
    }

    for frame in frame.clone() {
        if let Some(func) = frame.get_function(&cmd) {
            return CallType::Internal(func, args);
        }
    }

    let cmd = match shell.find_exe(&cmd) {
        Some(cmd) => cmd,
        None => cmd,
    };

    // TODO fix this stringification of args
    // it should flatten arrays
    // and throw error when it cannot convert it to a string
    let args: Vec<_> = args.into_iter().map(|v| v.to_string()).collect();
    CallType::External(P::new(Exec::cmd(cmd.clone()).args(&args)), cmd)
}

fn expand_call(
    shell: &mut Shell,
    frame: &mut Frame,
    commandparts: &[CommandPart],
    args: &[Argument],
    output: &mut OutputStream,
) -> Result<(String, Vec<Value>), ShellErrorKind> {
    let mut expanded_args = Vec::new();
    for arg in args {
        expanded_args.push(arg.eval(shell, frame, output)?);
    }

    let mut command = String::new();
    for part in commandparts.iter() {
        command.push_str(&part.eval(shell, frame, output)?);
    }

    if let Some(alias) = shell.aliases.get(&command) {
        let mut split = alias.split_whitespace();
        command = split.next().unwrap().to_string();
        let mut args: Vec<_> = split.map(|s| Value::from(s.to_string())).collect();
        args.extend(expanded_args.into_iter());
        expanded_args = args;
    }
    Ok((command, expanded_args))
}

pub enum CallType {
    Builtin(BulitinFn, Vec<Value>),
    Internal(Rc<(Vec<Variable>, Block)>, Vec<Value>),
    External(P<Exec>, String),
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
