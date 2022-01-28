use std::{
    collections::{HashMap, VecDeque},
    mem,
    rc::Rc,
    thread,
};

use subprocess::{CommunicateError, Exec, Pipeline, Redirection};

use crate::{
    parser::{
        ast::{Literal, Variable},
        runtime_error::RunTimeError,
        P,
    },
    shell::{
        builtins::{self, functions::BulitinFn},
        stream::{OutputStream, ValueStream},
        value::{Type, Value},
        Shell,
    },
};

pub mod binop;
use binop::BinOp;

pub mod unop;
use thin_string::{ThinString, ToThinString};
use unop::UnOp;

pub mod command;
use command::Command;

pub mod argument;
use argument::{Argument, ArgumentValue};

use super::Block;

// used to implement comparison operators without duplciating code
macro_rules! compare_impl {
    ($arg_lhs:expr, $arg_rhs:expr, $arg_binop:expr, $op:tt) => {{
        #[inline(always)]
        fn compare_impl_fn(lhs: Value, rhs: Value, binop: BinOp) -> Result<Value, RunTimeError> {
            match lhs.as_ref() {
                Value::Int(number) => match rhs.as_ref() {
                    Value::Int(rhs) => Ok(Value::Bool(number $op rhs)),
                    Value::Float(rhs) => Ok(Value::Bool((*number as f64) $op *rhs)),
                    Value::Bool(rhs) => Ok(Value::Bool(*number $op *rhs as i64)),
                    _ => Err(RunTimeError::InvalidBinaryOperand(
                        binop,
                        lhs.to_type(),
                        rhs.to_type(),
                    )),
                },
                Value::Float(number) => match rhs.as_ref() {
                    Value::Int(rhs) => Ok(Value::Bool(*number $op *rhs as f64)),
                    Value::Float(rhs) => Ok(Value::Bool(number $op rhs)),
                    Value::Bool(rhs) => Ok(Value::Bool(*number $op *rhs as i64 as f64)),
                    _ => Err(RunTimeError::InvalidBinaryOperand(
                        binop,
                        lhs.to_type(),
                        rhs.to_type(),
                    )),
                },
                Value::Bool(boolean) => match rhs.as_ref() {
                    Value::Int(rhs) => Ok(Value::Bool((*boolean as i64) $op *rhs)),
                    Value::Float(rhs) => Ok(Value::Bool((*boolean as i64 as f64) $op *rhs)),
                    Value::Bool(rhs) => Ok(Value::Bool(*boolean $op *rhs)),
                    _ => Err(RunTimeError::InvalidBinaryOperand(
                        binop,
                        lhs.to_type(),
                        rhs.to_type(),
                    )),
                },
                Value::String(string) => match rhs.as_ref() {
                    Value::String(rhs) => Ok(Value::Bool(string $op rhs)),
                    _ => Err(RunTimeError::InvalidBinaryOperand(
                        binop,
                        lhs.to_type(),
                        rhs.to_type(),
                    )),
                },
                _ => Err(RunTimeError::InvalidBinaryOperand(
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
    Call(Command, Vec<Argument>),
    Pipe(Vec<Expr>),
    Variable(Variable),
    Binary(BinOp, P<Expr>, P<Expr>),
    Paren(P<Expr>),
    Unary(UnOp, P<Expr>),
    Literal(Literal),
    TypeCast(Type, P<Expr>),
}

impl Expr {
    #[inline(always)]
    pub fn wrap(self, unop: Option<UnOp>) -> Self {
        match unop {
            Some(unop) => Expr::Unary(unop, P::new(self)),
            None => self,
        }
    }

    pub fn eval(&self, shell: &mut Shell, sub_expr: bool) -> Result<Value, RunTimeError> {
        match self {
            Self::Call(_, _) => {
                unreachable!("calls must always be in a pipeline, bare calls are not allowed")
            }
            Self::Literal(literal) => literal.eval(shell),
            Self::Variable(variable) => variable.eval(shell),
            // casting syntax will be reworked in favor of builtin
            Self::TypeCast(type_of, expr) => {
                let value = expr.eval(shell, false)?;
                match type_of {
                    Type::Int => match value {
                        Value::String(string) => Ok(Value::Int(string.parse()?)),
                        _ => Err(RunTimeError::InvalidConversion {
                            from: value.to_type(),
                            to: *type_of,
                        }),
                    },
                    Type::Float => match value {
                        Value::String(string) => Ok(Value::Float(string.parse()?)),
                        _ => Err(RunTimeError::InvalidConversion {
                            from: value.to_type(),
                            to: *type_of,
                        }),
                    },
                    Type::String => Ok(Value::String(value.to_string().to_thin_string())),
                    Type::List => match value {
                        Value::String(string) => Ok(Value::List(
                            string
                                .chars()
                                .map(|c| Value::String(ThinString::from(c)))
                                .collect(),
                        )),
                        Value::Range(range) => Ok(Value::List(
                            // clippy thinks Value::Int(n) is a function call
                            #[allow(clippy::redundant_closure)]
                            (*range)
                                .clone()
                                .into_iter()
                                .map(|n| Value::Int(n))
                                .collect(),
                        )),
                        _ => Err(RunTimeError::InvalidConversion {
                            from: value.to_type(),
                            to: *type_of,
                        }),
                    },
                    Type::Bool => Ok(Value::Bool(value.truthy())),
                    _ => unreachable!(),
                }
            }
            Self::Unary(unop, expr) => {
                let value = expr.eval(shell, false)?;
                match unop {
                    UnOp::Neg => match value.as_ref() {
                        Value::Int(int) => Ok(Value::Int(-*int)),
                        Value::Float(float) => Ok(Value::Float(-*float)),
                        _ => Err(RunTimeError::InvalidUnaryOperand(*unop, value.to_type())),
                    },
                    UnOp::Not => Ok(Value::Bool(!value.truthy())),
                }
            }
            Self::Paren(expr) => expr.eval(shell, false),
            Self::Binary(binop, lhs, rhs) => match binop {
                BinOp::Range => {
                    let lhs = lhs.eval(shell, false)?.try_as_int()?;
                    let rhs = rhs.eval(shell, false)?.try_as_int()?;
                    Ok(Value::Range(P::new(lhs..rhs)))
                }
                BinOp::Add => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;
                    lhs.try_add(rhs)
                }
                BinOp::Sub => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;
                    lhs.try_sub(rhs)
                }
                BinOp::Mul => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;
                    lhs.try_mul(rhs)
                }
                BinOp::Div => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;
                    lhs.try_div(rhs)
                }
                BinOp::Expo => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;
                    lhs.try_expo(rhs)
                }
                BinOp::Mod => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;
                    lhs.try_mod(rhs)
                }
                // The == operator (equality)
                BinOp::Eq => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;
                    Ok(Value::Bool(lhs == rhs))
                }
                // The != operator (not equal to)
                BinOp::Ne => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;
                    Ok(Value::Bool(lhs != rhs))
                }

                // all the ordering operators are the same except for the operator
                // the this is why the macro is used

                // The < operator (less than)
                BinOp::Lt => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;
                    compare_impl!(lhs, rhs, *binop, <)
                }
                // The <= operator (less than or equal to)
                BinOp::Le => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;
                    compare_impl!(lhs, rhs, *binop, <=)
                }
                // The >= operator (greater than or equal to)
                BinOp::Ge => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;
                    compare_impl!(lhs, rhs, *binop, >=)
                }
                // The > operator (greater than)
                BinOp::Gt => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;
                    compare_impl!(lhs, rhs, *binop, >)
                }
                BinOp::And => Ok(Value::Bool(
                    lhs.eval(shell, false)?.truthy() && rhs.eval(shell, false)?.truthy(),
                )),
                BinOp::Or => Ok(Value::Bool(
                    lhs.eval(shell, false)?.truthy() || rhs.eval(shell, false)?.truthy(),
                )),
            },
            Self::Pipe(calls) => {
                let mut expanded_calls = VecDeque::new();
                for callable in calls {
                    match callable {
                        Self::Call(cmd, args) => {
                            let (cmd, args) = expand_call(shell, cmd, args)?;
                            expanded_calls.push_back(get_call_type(shell, cmd, args));
                        }
                        _ => unreachable!(),
                    }
                }

                let mut execs = Vec::new();
                let mut output = OutputStream::default();
                while let Some(call_type) = expanded_calls.pop_front() {
                    match call_type {
                        CallType::External(exec) => {
                            execs.push(exec);
                        }
                        CallType::Builtin(builtin, args) => {
                            let stream = if execs.is_empty() {
                                let mut stream = ValueStream::default();
                                mem::swap(&mut output.stream, &mut stream);
                                stream
                            } else {
                                let value = Value::String(
                                    run_pipeline(shell, execs, true, output.stream)?
                                        .unwrap()
                                        .to_thin_string(),
                                );
                                execs = Vec::new();
                                ValueStream::from_value(value)
                            };
                            output = builtin(shell, &args, stream)?;
                        }
                        CallType::Internal(func, args) => {
                            let stream = if execs.is_empty() {
                                let mut stream = ValueStream::default();
                                mem::swap(&mut output.stream, &mut stream);
                                stream
                            } else {
                                let value = Value::String(
                                    run_pipeline(shell, execs, true, output.stream)?
                                        .unwrap()
                                        .to_thin_string(),
                                );
                                execs = Vec::new();
                                ValueStream::from_value(value)
                            };

                            let (vars, block) = &*func;
                            let mut input_vars = HashMap::new();
                            for (i, var) in vars.iter().enumerate() {
                                match args.get(i) {
                                    Some(arg) => {
                                        input_vars.insert(
                                            var.name.clone(),
                                            Value::String(arg.to_thin_string()),
                                        );
                                    }
                                    None => {
                                        return Err(RunTimeError::ToFewArguments {
                                            // this should be function name
                                            name: String::from("function"),
                                            expected: vars.len(),
                                            recived: args.len(),
                                        });
                                    }
                                }
                            }
                            output = block.eval(shell, Some(input_vars), Some(stream))?;
                        }
                    }
                }

                // capture output should be true if this is sub expr.
                if !execs.is_empty() {
                    if sub_expr {
                        let value = Value::String(
                            run_pipeline(shell, execs, true, output.stream)?
                                .unwrap()
                                .to_thin_string(),
                        );
                        Ok(Value::OutputStream(Box::new(OutputStream::new(
                            ValueStream::from_value(value),
                            0,
                        ))))
                    } else {
                        run_pipeline(shell, execs, false, output.stream)?;
                        Ok(Value::OutputStream(Box::new(OutputStream::default())))
                    }
                } else {
                    Ok(Value::OutputStream(Box::new(output)))
                }
            }
        }
    }
}

fn run_pipeline(
    shell: &mut Shell,
    mut execs: Vec<Exec>,
    capture_output: bool,
    input: ValueStream,
) -> Result<Option<String>, RunTimeError> {
    let mut input_string = String::new();
    for value in input {
        match value {
            Value::String(text) => {
                input_string.push_str(&text);
                input_string.push('\n');
            }
            _ => panic!("expected string input"),
        }
    }
    if execs.len() == 1 {
        let exec = if capture_output {
            execs.pop().unwrap().stdout(Redirection::Pipe)
        } else {
            execs.pop().unwrap()
        }
        .stdin(Redirection::Pipe);

        let mut child = exec.popen()?;
        shell.set_child(child.pid());
        if capture_output {
            let mut com = child.communicate_start(Some(input_string.into()));
            let t = thread::spawn::<_, Result<Option<String>, CommunicateError>>(move || {
                let (out, _) = com.read_string()?;
                Ok(out)
            });
            // the exit status should be set to the status variable
            let _status = child.wait()?;
            Ok(t.join().unwrap()?)
        } else {
            let _ = child.communicate_start(Some(input_string.into()));
            let _ = child.wait()?;
            Ok(None)
        }
    } else {
        let pipeline = Pipeline::from_exec_iter(execs).stdin(Redirection::Pipe);
        let mut children = pipeline.popen()?;
        children
            .first_mut()
            .unwrap()
            .communicate_bytes(Some(input_string.as_bytes()))?;
        let last = children.last_mut().unwrap();
        shell.set_child(last.pid());

        if capture_output {
            shell.set_child(None);
            Ok(None)
        } else {
            let _ = last.wait()?;
            shell.set_child(None);
            Ok(None)
        }
    }
}

fn get_call_type(shell: &Shell, cmd: String, args: Vec<String>) -> CallType {
    if let Some(builtin) = builtins::functions::get_builtin(&cmd) {
        return CallType::Builtin(builtin, args);
    }

    for frame in shell.stack.iter().rev() {
        if let Some(func) = frame.functions.get(&cmd) {
            return CallType::Internal(func.clone(), args);
        }
    }

    CallType::External(Exec::cmd(cmd).args(&args))
}

fn expand_call(
    shell: &mut Shell,
    command: &Command,
    args: &[Argument],
) -> Result<(String, Vec<String>), RunTimeError> {
    let mut expanded_args = Vec::new();
    for arg in args {
        let arg = arg.eval(shell)?;
        match arg {
            ArgumentValue::Single(string) => expanded_args.push(string),
            ArgumentValue::Multi(vec) => expanded_args.extend(vec.into_iter()),
        }
    }
    let mut command = command.eval(shell)?;

    if let Some(alias) = shell.aliases.get(&command) {
        let mut split = alias.split_whitespace();
        command = split.next().unwrap().to_string();
        let mut args: Vec<_> = split.map(|s| s.to_string()).collect();
        args.extend(expanded_args.into_iter());
        expanded_args = args;
    }
    Ok((command, expanded_args))
}

pub enum CallType {
    Builtin(BulitinFn, Vec<String>),
    Internal(Rc<(Vec<Variable>, Block)>, Vec<String>),
    External(Exec),
}
