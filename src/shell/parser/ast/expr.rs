use thin_vec::*;

use crate::{
    parser::{
        ast::{Direction, Literal, Variable},
        runtime_error::RunTimeError,
        P,
    },
    shell::{
        builtins,
        values::{HeapValue, Value, ValueKind},
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
pub enum Type {
    Int,
    Float,
    Str,
    Bool,
}

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
    TypeCast(Type, P<Expr>),
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
                Ok(Value::Int(0).into())
            }
            Self::Literal(literal) => literal.eval(shell),
            Self::Variable(variable) => variable.eval(shell),
            Self::TypeCast(type_of, expr) => {
                let value = expr.eval(shell, false)?;
                match type_of {
                    Type::Int => todo!(),
                    Type::Float => todo!(),
                    Type::Str => Ok(Value::String(value.to_string().to_thin_string()).into()),
                    Type::Bool => Ok(Value::Bool(value.truthy()).into()),
                }
            }
            Self::Unary(unop, expr) => {
                let value = expr.eval(shell, false)?;
                match unop {
                    UnOp::Neg => match value.as_ref() {
                        Value::Int(int) => Ok(Value::Int(*int).into()),
                        Value::Float(float) => Ok(Value::Float(*float).into()),
                        _ => return Err(RunTimeError::InvalidOperand),
                    },
                    UnOp::Not => Ok(Value::Bool(!value.truthy()).into()),
                }
            }
            Self::Paren(expr) => expr.eval(shell, false),
            Self::Binary(binop, lhs, rhs) => match binop {
                BinOp::Range => {
                    let lhs = lhs.eval(shell, false)?.try_as_int()?;
                    let rhs = rhs.eval(shell, false)?.try_as_int()?;
                    Ok(Value::Range(P::new(lhs..rhs)).into())
                }
                BinOp::Add => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;

                    match lhs.as_ref() {
                        Value::Int(number) => match rhs.as_ref() {
                            Value::List(rhs) => {
                                let mut list: ThinVec<HeapValue> = thin_vec![lhs.clone().into()];
                                list.extend(rhs.iter().map(|value| value.clone().into()));
                                return Ok(Value::List(list).into());
                            }
                            Value::String(string) => {
                                let mut thin_string = number.to_thin_string();
                                thin_string.push_str(string);
                                return Ok(Value::String(thin_string).into());
                            }
                            Value::Float(rhs) => {
                                return Ok(Value::Float(*number as f64 + *rhs).into())
                            }
                            _ => Ok(Value::Int(number + lhs.try_as_int()?).into()),
                        },
                        Value::Float(number) => match rhs.as_ref() {
                            Value::List(rhs) => {
                                let mut list: ThinVec<HeapValue> = thin_vec![lhs.clone().into()];
                                list.extend(rhs.iter().map(|value| value.clone().into()));
                                return Ok(Value::List(list).into());
                            }
                            Value::String(string) => {
                                let mut thin_string = number.to_thin_string();
                                thin_string.push_str(string);
                                return Ok(Value::String(thin_string).into());
                            }
                            _ => Ok(Value::Float(number + lhs.try_as_float()?).into()),
                        },
                        Value::Bool(boolean) => match rhs.as_ref() {
                            Value::List(rhs) => {
                                let mut list: ThinVec<HeapValue> = thin_vec![lhs.clone().into()];
                                list.extend(rhs.iter().map(|value| value.clone().into()));
                                return Ok(Value::List(list).into());
                            }
                            Value::Float(rhs) => {
                                return Ok(Value::Float(*boolean as i64 as f64 + *rhs).into())
                            }
                            Value::String(string) => {
                                let mut thin_string = boolean.to_thin_string();
                                thin_string.push_str(string);
                                return Ok(Value::String(thin_string).into());
                            }
                            _ => Ok(Value::Int(*boolean as i64 + lhs.try_as_int()?).into()),
                        },
                        Value::String(string) => {
                            if let Value::List(rhs) = rhs.as_ref() {
                                let mut list: ThinVec<HeapValue> = thin_vec![lhs.clone().into()];
                                list.extend(rhs.iter().map(|value| value.clone().into()));
                                return Ok(Value::List(list).into());
                            }

                            let mut new = string.clone();
                            let rhs = rhs.to_string();
                            new.push_str(&rhs);
                            Ok(Value::String(new).into())
                        }
                        Value::List(lhs) => {
                            let mut list = lhs.clone();
                            list.push(rhs.into());
                            Ok(Value::List(list).into())
                        }
                        _ => Err(RunTimeError::InvalidOperand),
                    }
                }
                BinOp::Sub => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;

                    match lhs.as_ref() {
                        Value::Int(number) => match rhs.as_ref() {
                            Value::Int(rhs) => Ok(Value::Int(number - rhs).into()),
                            Value::Float(rhs) => Ok(Value::Float(*number as f64 - rhs).into()),
                            _ => Err(RunTimeError::InvalidOperand),
                        },
                        Value::Float(number) => {
                            Ok(Value::Float(*number as f64 - rhs.try_as_float()?).into())
                        }
                        Value::Bool(boolean) => match rhs.as_ref() {
                            Value::Int(rhs) => Ok(Value::Int(*boolean as i64 - rhs).into()),
                            Value::Float(rhs) => {
                                Ok(Value::Float(*boolean as i64 as f64 - rhs).into())
                            }
                            _ => Err(RunTimeError::InvalidOperand),
                        },
                        _ => Err(RunTimeError::InvalidOperand),
                    }
                }
                BinOp::Mul => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;

                    match lhs.as_ref() {
                        Value::Int(number) => match rhs.as_ref() {
                            Value::Int(rhs) => Ok(Value::Int(number * rhs).into()),
                            Value::Float(rhs) => Ok(Value::Float(*number as f64 * rhs).into()),
                            Value::String(string) => {
                                let mut new = ThinString::new();
                                for _ in 0..*number {
                                    new.push_str(string);
                                }
                                Ok(Value::String(new).into())
                            }
                            _ => Err(RunTimeError::InvalidOperand),
                        },
                        Value::Float(number) => {
                            Ok(Value::Float(*number as f64 * rhs.try_as_float()?).into())
                        }
                        Value::Bool(boolean) => match rhs.as_ref() {
                            Value::Int(rhs) => Ok(Value::Int(*boolean as i64 * rhs).into()),
                            Value::Float(rhs) => {
                                Ok(Value::Float(*boolean as i64 as f64 * rhs).into())
                            }
                            Value::String(string) => {
                                let mut new = ThinString::new();
                                for _ in 0..*boolean as i64 {
                                    new.push_str(string);
                                }
                                Ok(Value::String(new).into())
                            }
                            _ => Err(RunTimeError::InvalidOperand),
                        },
                        Value::String(string) => {
                            let mut new = ThinString::new();
                            let mul = rhs.try_as_int()?;
                            for _ in 0..mul {
                                new.push_str(string);
                            }
                            Ok(Value::String(new).into())
                        }
                        Value::List(list) => {
                            let mut new = ThinVec::new();
                            let mul = rhs.try_as_int()?;
                            for _ in 0..mul {
                                new.extend_from_slice(list);
                            }
                            Ok(Value::List(new).into())
                        }
                        _ => Err(RunTimeError::InvalidOperand),
                    }
                }
                BinOp::Div => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;

                    match lhs.as_ref() {
                        Value::Int(number) => {
                            Ok(Value::Float(*number as f64 / rhs.try_as_float()?).into())
                        }
                        Value::Float(number) => {
                            Ok(Value::Float(*number as f64 / rhs.try_as_float()?).into())
                        }
                        Value::Bool(boolean) => {
                            Ok(Value::Float(*boolean as i64 as f64 / rhs.try_as_float()?).into())
                        }
                        _ => Err(RunTimeError::InvalidOperand),
                    }
                }
                BinOp::Expo => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;

                    match lhs.as_ref() {
                        Value::Int(number) => {
                            Ok(Value::Float((*number as f64).powf(rhs.try_as_float()?)).into())
                        }
                        Value::Float(number) => {
                            Ok(Value::Float((*number as f64).powf(rhs.try_as_float()?)).into())
                        }
                        Value::Bool(boolean) => Ok(Value::Float(
                            (*boolean as i64 as f64).powf(rhs.try_as_float()?),
                        )
                        .into()),
                        _ => Err(RunTimeError::InvalidOperand),
                    }
                }
                BinOp::Mod => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;

                    match lhs.as_ref() {
                        Value::Int(number) => match rhs.as_ref() {
                            Value::Int(rhs) => Ok(Value::Int(number % rhs).into()),
                            Value::Float(rhs) => Ok(Value::Float(*number as f64 % rhs).into()),
                            _ => Err(RunTimeError::InvalidOperand),
                        },
                        Value::Float(number) => {
                            Ok(Value::Float(*number as f64 % rhs.try_as_float()?).into())
                        }
                        Value::Bool(boolean) => match rhs.as_ref() {
                            Value::Int(rhs) => Ok(Value::Int(*boolean as i64 % rhs).into()),
                            Value::Float(rhs) => {
                                Ok(Value::Float(*boolean as i64 as f64 % rhs).into())
                            }
                            _ => Err(RunTimeError::InvalidOperand),
                        },
                        _ => Err(RunTimeError::InvalidOperand),
                    }
                }
                // The == operator (equality)
                BinOp::Eq => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;

                    Ok(Value::Bool(lhs == rhs).into())
                }
                // The < operator (less than)
                BinOp::Lt => todo!(),
                // The <= operator (less than or equal to)
                BinOp::Le => todo!(),
                // The != operator (not equal to)
                BinOp::Ne => todo!(),
                // The >= operator (greater than or equal to)
                BinOp::Ge => todo!(),
                // The > operator (greater than)
                BinOp::Gt => todo!(),
                BinOp::And => todo!(),
                BinOp::Or => todo!(),
            },
            _ => todo!(),
        }
    }
}
