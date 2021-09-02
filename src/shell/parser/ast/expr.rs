use thin_vec::ThinVec;

use crate::{
    parser::{
        ast::{Literal, Variable},
        runtime_error::RunTimeError,
        P,
    },
    shell::{builtins, gc::Value, Shell},
};

pub mod binop;
use binop::BinOp;

pub mod unop;
use thin_string::ToThinString;
use unop::UnOp;

pub mod command;
use command::Command;

pub mod argument;
use argument::{Argument, ArgumentValue};

#[derive(Debug)]
pub enum Direction {
    Left,
    Right,
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
}

impl Expr {
    #[inline]
    pub fn wrap(self, unop: Option<UnOp>) -> Self {
        match unop {
            Some(unop) => Expr::Unary(unop, P::new(self)),
            None => self,
        }
    }

    pub fn eval(&self, shell: &mut Shell) -> Result<Value, RunTimeError> {
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
                    return res;
                } else {
                    match shell.execute_command(&command, &expanded_args) {
                        Ok(_) => (),
                        Err(error) => match error.kind() {
                            std::io::ErrorKind::NotFound => {
                                return Err(RunTimeError::CommandNotFound(command))
                            }
                            _ => return Err(RunTimeError::IoError(error)),
                        },
                    }
                }
                Ok(Value::ExitStatus(0))
            }
            Self::Literal(literal) => match literal {
                Literal::String(string) => Ok(Value::String(string.to_thin_string())),
                Literal::Expand(_expand) => todo!(),
                Literal::List(list) => {
                    let mut values = ThinVec::new();
                    for expr in list.iter() {
                        values.push(expr.eval(shell)?);
                    }
                    Ok(Value::List(values))
                }
                Literal::Float(number) => Ok(Value::Int(*number as i64)),
                Literal::Int(number) => Ok(Value::Int(*number as i64)),
                Literal::Bool(boolean) => Ok(Value::Bool(*boolean)),
            },
            _ => todo!(),
        }
    }
}
