use std::convert::TryFrom;

use bigdecimal::num_bigint::BigUint;
use num_traits::cast::ToPrimitive;

use crate::{
    parser::{
        ast::{expr::argument::Expand, Expr},
        runtime_error::RunTimeError,
        syntax_error::SyntaxErrorKind,
        Token, TokenType,
    },
    shell::{value::Value, Shell},
};

#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Expand(Expand),
    List(Vec<Expr>),
    Float(f64),
    Int(BigUint),
    Bool(bool),
}

impl Literal {
    pub fn eval(&self, shell: &mut Shell) -> Result<Value, RunTimeError> {
        match self {
            Literal::String(string) => Ok(Value::String(string.to_string())),
            Literal::Expand(expand) => Ok(Value::String(expand.eval(shell)?)),
            Literal::List(list) => {
                let mut values: Vec<Value> = Vec::new();
                for expr in list.iter() {
                    let value = expr.eval(shell, false)?;
                    match value {
                        Value::List(ref list) => {
                            for item in list {
                                values.push(item.clone());
                            }
                        }
                        _ => values.push(value),
                    }
                }
                Ok(Value::List(values))
            }
            Literal::Float(number) => Ok(Value::Float(*number)),
            Literal::Int(number) => match number.to_i64() {
                Some(number) => Ok(Value::Int(number)),
                None => Err(RunTimeError::IntegerOverFlow),
            },
            Literal::Bool(boolean) => Ok(Value::Bool(*boolean)),
        }
    }
}

impl TryFrom<Token> for Literal {
    type Error = SyntaxErrorKind;
    fn try_from(token: Token) -> Result<Self, SyntaxErrorKind> {
        match token.token_type {
            TokenType::String(text) => Ok(Literal::String(text)),
            TokenType::Float(number, _) => Ok(Literal::Float(number)),
            TokenType::Int(number, _) => Ok(Literal::Int(number)),
            TokenType::True => Ok(Literal::Bool(true)),
            TokenType::False => Ok(Literal::Bool(false)),
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }
}
