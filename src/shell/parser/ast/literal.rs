use std::{convert::TryFrom, rc::Rc};

use bigdecimal::{num_bigint::BigUint, BigDecimal};
use indexmap::IndexMap;
use num_traits::cast::ToPrimitive;
use regex::Regex;

use super::context::Context;
use crate::{
    parser::{
        ast::{expr::argument::Expand, Expr},
        lexer::token::span::Span,
        shell_error::ShellErrorKind,
        syntax_error::SyntaxErrorKind,
        Token, TokenType,
    },
    shell::value::{table::Table, SpannedValue, Value},
};

#[derive(Debug, Clone)]
pub enum LiteralKind {
    String(Rc<String>),
    Expand(Expand),
    List(Vec<Expr>),
    Map(Vec<(Expr, Expr)>),
    Float(BigDecimal),
    Int(BigUint),
    Bool(bool),
    Regex(Rc<(Regex, String)>),
}

impl LiteralKind {
    pub fn spanned(self, span: Span) -> Literal {
        Literal { kind: self, span }
    }
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub kind: LiteralKind,
    pub span: Span,
}

impl TryFrom<Token> for Literal {
    type Error = SyntaxErrorKind;
    fn try_from(token: Token) -> Result<Self, SyntaxErrorKind> {
        match token.token_type {
            TokenType::Float(number, _) => Ok(LiteralKind::Float(number).spanned(token.span)),
            TokenType::Int(number, _) => Ok(LiteralKind::Int(number).spanned(token.span)),
            TokenType::True => Ok(LiteralKind::Bool(true).spanned(token.span)),
            TokenType::False => Ok(LiteralKind::Bool(false).spanned(token.span)),
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }
}

impl Literal {
    pub fn eval(&self, ctx: &mut Context) -> Result<SpannedValue, ShellErrorKind> {
        let span = self.span;
        match &self.kind {
            LiteralKind::String(string) => Ok(Value::from(string.to_string()).spanned(span)),
            LiteralKind::Expand(expand) => Ok(Value::from(expand.eval(ctx)?).spanned(span)),
            LiteralKind::List(list) => {
                let mut values: Vec<Value> = Vec::new();
                let mut is_table = true;
                for expr in list.iter() {
                    values.push(expr.eval(ctx)?.into());
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
                    Ok(Value::from(table).spanned(span))
                } else {
                    Ok(Value::from(values).spanned(span))
                }
            }
            LiteralKind::Map(exprs) => {
                let mut map = IndexMap::new();
                for (key, value) in exprs {
                    let key = key.eval(ctx)?.try_into_string()?;
                    let value = value.eval(ctx)?.into();
                    map.insert(key, value);
                }
                Ok(Value::Map(Rc::new(map)).spanned(span))
            }
            LiteralKind::Float(number) => Ok(Value::Float(number.to_f64().unwrap()).spanned(span)),
            LiteralKind::Int(number) => match number.to_i64() {
                Some(number) => Ok(Value::Int(number).spanned(span)),
                None => Err(ShellErrorKind::IntegerOverFlow),
            },
            LiteralKind::Bool(boolean) => Ok(Value::Bool(*boolean).spanned(span)),
            LiteralKind::Regex(regex) => Ok(Value::Regex(regex.clone()).spanned(span)),
        }
    }
}
