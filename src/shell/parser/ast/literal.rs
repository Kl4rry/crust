use std::{convert::TryFrom, rc::Rc};

use bigdecimal::{num_bigint::BigUint, BigDecimal};
use indexmap::IndexMap;
use num_traits::cast::ToPrimitive;
use regex::Regex;

use crate::{
    parser::{
        ast::{expr::argument::Expand, Expr},
        shell_error::ShellErrorKind,
        syntax_error::SyntaxErrorKind,
        Token, TokenType,
    },
    shell::{
        frame::Frame,
        stream::OutputStream,
        value::{table::Table, Value},
        Shell,
    },
};

#[derive(Debug, Clone)]
pub enum Literal {
    String(Rc<String>),
    Expand(Expand),
    List(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
    Float(BigDecimal),
    Int(BigUint),
    Bool(bool),
    Regex(Rc<(Regex, String)>),
}

impl Literal {
    pub fn eval(
        &self,
        shell: &mut Shell,
        frame: &mut Frame,
        output: &mut OutputStream,
    ) -> Result<Value, ShellErrorKind> {
        match self {
            Literal::String(string) => Ok(Value::from(string.to_string())),
            Literal::Expand(expand) => Ok(Value::from(expand.eval(shell, frame, output)?)),
            Literal::List(list) => {
                let mut values: Vec<Value> = Vec::new();
                let mut is_table = true;
                for expr in list.iter() {
                    values.push(expr.eval(shell, frame, output)?);
                    unsafe {
                        if !matches!(values.last().unwrap_unchecked(), Value::Map(_)) {
                            is_table = false;
                        }
                    }
                }

                if is_table {
                    let mut table = Table::new();
                    for value in values {
                        table.insert_map(Rc::unwrap_or_clone(value.unwrap_map()));
                    }
                    Ok(Value::from(table))
                } else {
                    Ok(Value::from(values))
                }
            }
            Literal::Map(exprs) => {
                let mut map = IndexMap::new();
                for (key, value) in exprs {
                    let key = key.eval(shell, frame, output)?.try_into_string()?;
                    let value = value.eval(shell, frame, output)?;
                    map.insert(key, value);
                }
                Ok(Value::Map(Rc::new(map)))
            }
            Literal::Float(number) => Ok(Value::Float(number.to_f64().unwrap())),
            Literal::Int(number) => match number.to_i64() {
                Some(number) => Ok(Value::Int(number)),
                None => Err(ShellErrorKind::IntegerOverFlow),
            },
            Literal::Bool(boolean) => Ok(Value::Bool(*boolean)),
            Literal::Regex(regex) => Ok(Value::Regex(regex.clone())),
        }
    }
}

impl TryFrom<Token> for Literal {
    type Error = SyntaxErrorKind;
    fn try_from(token: Token) -> Result<Self, SyntaxErrorKind> {
        match token.token_type {
            TokenType::String(text) => Ok(Literal::String(Rc::new(text))),
            TokenType::Float(number, _) => Ok(Literal::Float(number)),
            TokenType::Int(number, _) => Ok(Literal::Int(number)),
            TokenType::True => Ok(Literal::Bool(true)),
            TokenType::False => Ok(Literal::Bool(false)),
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }
}
