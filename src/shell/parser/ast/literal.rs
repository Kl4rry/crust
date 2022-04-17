use std::{collections::HashMap, convert::TryFrom};

use bigdecimal::{num_bigint::BigUint, BigDecimal};
use num_traits::cast::ToPrimitive;

use crate::{
    parser::{
        ast::{expr::argument::Expand, Expr},
        shell_error::ShellErrorKind,
        syntax_error::SyntaxErrorKind,
        Token, TokenType, P,
    },
    shell::{stream::OutputStream, value::Value, Shell},
};

#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Expand(Expand),
    List(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
    Float(BigDecimal),
    Int(BigUint),
    Bool(bool),
}

impl Literal {
    pub fn eval(
        &self,
        shell: &mut Shell,
        output: &mut OutputStream,
    ) -> Result<Value, ShellErrorKind> {
        match self {
            Literal::String(string) => Ok(Value::String(string.to_string())),
            Literal::Expand(expand) => Ok(Value::String(expand.eval(shell, output)?)),
            Literal::List(list) => {
                let mut values: Vec<Value> = Vec::new();
                for expr in list.iter() {
                    values.push(expr.eval(shell, output)?);
                }
                Ok(Value::List(values))
            }
            Literal::Map(exprs) => {
                let mut map = HashMap::new();
                for (key, value) in exprs {
                    let key = key.eval(shell, output)?.try_as_string()?;
                    let value = value.eval(shell, output)?;
                    map.insert(key, value);
                }
                Ok(Value::Map(P::new(map)))
            }
            Literal::Float(number) => Ok(Value::Float(number.to_f64().unwrap())),
            Literal::Int(number) => match number.to_i128() {
                Some(number) => Ok(Value::Int(number)),
                None => Err(ShellErrorKind::IntegerOverFlow),
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
