use crate::{
    parser::{
        ast::{Direction, Literal, Variable},
        runtime_error::RunTimeError,
        P,
    },
    shell::{
        builtins,
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
    #[inline(always)]
    pub fn wrap(self, unop: Option<UnOp>) -> Self {
        match unop {
            Some(unop) => Expr::Unary(unop, P::new(self)),
            None => self,
        }
    }

    pub fn eval(&self, shell: &mut Shell, piped: bool) -> Result<Value, RunTimeError> {
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

                if let Some(res) = builtins::functions::run_builtin(shell, &command, &expanded_args)
                {
                    return res;
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
                Ok(Value::Int(0))
            }
            Self::Literal(literal) => literal.eval(shell),
            Self::Variable(variable) => variable.eval(shell),
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
                BinOp::And => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;
                    Ok(Value::Bool(lhs.truthy() && rhs.truthy()))
                }
                BinOp::Or => {
                    let lhs = lhs.eval(shell, false)?;
                    let rhs = rhs.eval(shell, false)?;
                    Ok(Value::Bool(lhs.truthy() || rhs.truthy()))
                }
            },
            Self::Pipe(_lhs, _rhs) => todo!("pipe not impl"),
            Self::Redirect(_direction, _expr, _file) => todo!("redirect not impl"),
        }
    }
}
