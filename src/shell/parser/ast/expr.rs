use thin_vec::ThinVec;

use crate::{
    parser::{
        ast::{Direction, Literal, Variable},
        runtime_error::RunTimeError,
        P,
    },
    shell::{
        builtins,
        gc::{Value, ValueKind},
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

#[derive(Debug)]
pub enum Expr {
    Call(Command, Vec<Argument>),
    Pipe(P<Expr>, P<Expr>),
    Redirect(Direction, P<Expr>, Argument),
    Variable(Variable),
    Binary(BinOp, P<Expr>, P<Expr>),
    Paren(P<Expr>),
    Unary(UnOp, P<Expr>),
    Literal(Literal),
}

impl Expr {
    #[inline]
    pub fn wrap(self, unop: Option<UnOp>) -> Self {
        match unop {
            Some(unop) => Expr::Unary(unop, P::new(self)),
            None => self,
        }
    }

    pub fn eval(&self, shell: &mut Shell, piped: bool) -> Result<ValueKind, RunTimeError> {
        match self {
            Self::Call(command, args) => {
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

                if let Some(res) = builtins::run_builtin(shell, &command, &expanded_args) {
                    return Ok(res?.into());
                } else {
                    match shell.execute_command(&command, &expanded_args, piped) {
                        Ok(_) => (),
                        Err(error) => match error.kind() {
                            std::io::ErrorKind::NotFound => {
                                return Err(RunTimeError::CommandNotFound(command))
                            }
                            _ => return Err(RunTimeError::IoError(error)),
                        },
                    }
                }
                Ok(Value::ExitStatus(0).into())
            }
            Self::Literal(literal) => match literal {
                Literal::String(string) => Ok(Value::String(string.to_thin_string()).into()),
                Literal::Expand(_expand) => todo!(),
                Literal::List(list) => {
                    let mut values = ThinVec::new();
                    for expr in list.iter() {
                        values.push(expr.eval(shell, false)?);
                    }
                    Ok(Value::List(values).into())
                }
                Literal::Float(number) => Ok(Value::Float(*number).into()),
                Literal::Int(number) => Ok(Value::Int(*number as i64).into()),
                Literal::Bool(boolean) => Ok(Value::Bool(*boolean).into()),
            },
            Self::Variable(Variable { name }) => match shell.variables.get(name) {
                Some(value) => Ok(value.clone().into()),
                None => Err(RunTimeError::VariableNotFound),
            },
            Self::Unary(unop, expr) => {
                let value = expr.eval(shell, false)?;
                match unop {
                    UnOp::Neg => {
                        if value.as_ref().is_float() {
                            Ok(Value::Float(-value.as_ref().try_to_float()?).into())
                        } else {
                            Ok(Value::Int(-value.as_ref().try_to_int()?).into())
                        }
                    }
                    UnOp::Not => Ok(Value::Bool(!value.as_ref().try_to_bool()?).into()),
                }
            }
            Self::Paren(expr) => expr.eval(shell, false),
            Self::Binary(binop, lhs, rhs) => match binop {
                BinOp::Add => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;

                    match lhs.as_ref() {
                        Value::Int(number) => {
                            if rhs.as_ref().is_float() {
                                Ok(Value::Float(*number as f64 + rhs.as_ref().try_to_float()?)
                                    .into())
                            } else {
                                Ok(Value::Int(*number + rhs.as_ref().try_to_int()?).into())
                            }
                        }
                        Value::Float(number) => {
                            Ok(Value::Float(*number as f64 + rhs.as_ref().try_to_float()?).into())
                        }
                        Value::Bool(boolean) => {
                            if rhs.as_ref().is_float() {
                                Ok(Value::Float(
                                    *boolean as i64 as f64 + rhs.as_ref().try_to_float()?,
                                )
                                .into())
                            } else {
                                Ok(Value::Int(*boolean as i64 + rhs.as_ref().try_to_int()?).into())
                            }
                        }
                        Value::String(lhs) => {
                            let mut new = lhs.clone();
                            let rhs = rhs.as_ref().try_to_string()?;
                            new.push_str(&rhs);
                            Ok(Value::String(new).into())
                        }
                        _ => todo!(),
                    }
                }
                BinOp::Sub => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;

                    match lhs.as_ref() {
                        Value::Int(number) => {
                            if rhs.as_ref().is_float() {
                                Ok(Value::Float(*number as f64 - rhs.as_ref().try_to_float()?)
                                    .into())
                            } else {
                                Ok(Value::Int(*number - rhs.as_ref().try_to_int()?).into())
                            }
                        }
                        Value::Float(number) => {
                            Ok(Value::Float(*number as f64 - rhs.as_ref().try_to_float()?).into())
                        }
                        Value::Bool(boolean) => {
                            if rhs.as_ref().is_float() {
                                Ok(Value::Float(
                                    *boolean as i64 as f64 - rhs.as_ref().try_to_float()?,
                                )
                                .into())
                            } else {
                                Ok(Value::Int(*boolean as i64 - rhs.as_ref().try_to_int()?).into())
                            }
                        }
                        _ => todo!(),
                    }
                }
                BinOp::Mul => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;

                    match lhs.as_ref() {
                        Value::Int(number) => {
                            if rhs.as_ref().is_float() {
                                Ok(Value::Float(*number as f64 * rhs.as_ref().try_to_float()?)
                                    .into())
                            } else {
                                Ok(Value::Int(*number * rhs.as_ref().try_to_int()?).into())
                            }
                        }
                        Value::Float(number) => {
                            Ok(Value::Float(*number as f64 * rhs.as_ref().try_to_float()?).into())
                        }
                        Value::Bool(boolean) => {
                            if rhs.as_ref().is_float() {
                                Ok(Value::Float(
                                    *boolean as i64 as f64 * rhs.as_ref().try_to_float()?,
                                )
                                .into())
                            } else {
                                Ok(Value::Int(*boolean as i64 * rhs.as_ref().try_to_int()?).into())
                            }
                        }
                        Value::String(lhs) => {
                            let mut new = ThinString::new();
                            let mul = rhs.as_ref().try_to_int()?;
                            for _ in 0..mul {
                                new.push_str(lhs);
                            }
                            Ok(Value::String(new).into())
                        }
                        _ => todo!(),
                    }
                }
                BinOp::Div => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;

                    match lhs.as_ref() {
                        Value::Int(number) => {
                            if rhs.as_ref().is_float() {
                                Ok(Value::Float(*number as f64 / rhs.as_ref().try_to_float()?)
                                    .into())
                            } else {
                                Ok(Value::Int(*number / rhs.as_ref().try_to_int()?).into())
                            }
                        }
                        Value::Float(number) => {
                            Ok(Value::Float(*number as f64 / rhs.as_ref().try_to_float()?).into())
                        }
                        Value::Bool(boolean) => {
                            if rhs.as_ref().is_float() {
                                Ok(Value::Float(
                                    *boolean as i64 as f64 / rhs.as_ref().try_to_float()?,
                                )
                                .into())
                            } else {
                                Ok(Value::Int(*boolean as i64 / rhs.as_ref().try_to_int()?).into())
                            }
                        }
                        _ => todo!(),
                    }
                }
                BinOp::Expo => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;

                    match lhs.as_ref() {
                        Value::Int(number) => {
                            if rhs.as_ref().is_float() {
                                Ok(Value::Float(
                                    (*number as f64).powf(rhs.as_ref().try_to_float()?),
                                )
                                .into())
                            } else {
                                Ok(Value::Int(
                                    (*number as f64).powi(rhs.as_ref().try_to_int()? as i32) as i64,
                                )
                                .into())
                            }
                        }
                        Value::Float(number) => Ok(Value::Float(
                            (*number as f64).powf(rhs.as_ref().try_to_float()?),
                        )
                        .into()),
                        Value::Bool(boolean) => {
                            if rhs.as_ref().is_float() {
                                Ok(Value::Float(
                                    (*boolean as i64 as f64).powf(rhs.as_ref().try_to_float()?),
                                )
                                .into())
                            } else {
                                Ok(Value::Int(
                                    (*boolean as i64 as f64).powi(rhs.as_ref().try_to_int()? as i32)
                                        as i64,
                                )
                                .into())
                            }
                        }
                        _ => todo!(),
                    }
                }
                BinOp::Mod => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;

                    match lhs.as_ref() {
                        Value::Int(number) => {
                            if rhs.as_ref().is_float() {
                                Ok(Value::Float(*number as f64 % rhs.as_ref().try_to_float()?)
                                    .into())
                            } else {
                                Ok(Value::Int(*number % rhs.as_ref().try_to_int()?).into())
                            }
                        }
                        Value::Float(number) => {
                            Ok(Value::Float(*number as f64 % rhs.as_ref().try_to_float()?).into())
                        }
                        Value::Bool(boolean) => {
                            if rhs.as_ref().is_float() {
                                Ok(Value::Float(
                                    *boolean as i64 as f64 % rhs.as_ref().try_to_float()?,
                                )
                                .into())
                            } else {
                                Ok(Value::Int(*boolean as i64 % rhs.as_ref().try_to_int()?).into())
                            }
                        }
                        _ => todo!(),
                    }
                }
                _ => todo!(),
            },
            _ => todo!(),
        }
    }
}
