use std::convert::TryFrom;

use thin_string::ToThinString;
use thin_vec::ThinVec;

use crate::{
    parser::{
        ast::{expr::argument::Expand, Expr},
        runtime_error::RunTimeError,
        syntax_error::SyntaxErrorKind,
        Token, TokenType,
    },
    shell::{
        values::{HeapValue, Value, ValueKind},
        Shell,
    },
};

#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Expand(Expand),
    List(Vec<Expr>),
    Float(f64),
    Int(u128),
    Bool(bool),
}

impl Literal {
    pub fn eval(&self, shell: &mut Shell) -> Result<ValueKind, RunTimeError> {
        match self {
            Literal::String(string) => Ok(Value::String(string.to_thin_string()).into()),
            Literal::Expand(expand) => {
                Ok(Value::String(expand.eval(shell)?.to_thin_string()).into())
            }

            Literal::List(list) => {
                let mut values: ThinVec<HeapValue> = ThinVec::new();
                for expr in list.iter() {
                    let value = expr.eval(shell, false)?;
                    match *value {
                        Value::List(ref list) => {
                            for item in list {
                                values.push(item.clone());
                            }
                        }
                        _ => values.push(value.into()),
                    }
                }
                Ok(Value::List(values).into())
            }
            Literal::Float(number) => Ok(Value::Float(*number).into()),
            Literal::Int(number) => Ok(Value::Int(*number as i64).into()),
            Literal::Bool(boolean) => Ok(Value::Bool(*boolean).into()),
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
