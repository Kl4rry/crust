use crate::{
    parser::{
        ast::{Literal, Variable},
        runtime_error::RunTimeError,
        P,
    },
    shell::{gc::Value, Shell},
};

pub mod binop;
use binop::BinOp;

pub mod unop;
use unop::UnOp;

pub mod command;
use command::Command;

pub mod argument;
use argument::Argument;

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

    pub fn eval(&mut self, shell: &mut Shell) -> Result<Value, RunTimeError> {
        match self {
            Expr::Call(command, args) => {
                let mut expanded_args = Vec::new();
                for arg in args {
                    expanded_args.extend(arg.eval(shell)?.into_iter());
                }
                println!("command: {}", command.eval(shell)?);
                println!("args: {:?}", expanded_args);
            }
            _ => todo!(),
        }
        Ok(Value::ExitStatus(0))
    }
}
