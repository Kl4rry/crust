use bigdecimal::{num_bigint::BigInt, BigDecimal, ToPrimitive};

use crate::{
    parser::{
        ast::context::Context, lexer::token::span::Span, shell_error::ShellErrorKind, Expr,
        Variable,
    },
    shell::value::Value,
};

#[derive(Debug, Clone)]
pub struct ArgumentPart {
    pub kind: ArgumentPartKind,
    pub span: Span,
}

impl ArgumentPart {
    pub fn eval(&self, ctx: &mut Context) -> Result<Value, ShellErrorKind> {
        match &self.kind {
            ArgumentPartKind::Variable(var) => Ok(var.eval(ctx)?),
            ArgumentPartKind::Expand(expand) => Ok(Value::from(expand.eval(ctx)?)),
            ArgumentPartKind::Bare(value) => Ok(Value::from(value.to_string())),
            ArgumentPartKind::Quoted(string) => Ok(Value::from(string.clone())),
            ArgumentPartKind::Expr(expr) => Ok(expr.eval(ctx)?),
            ArgumentPartKind::Float(number) => Ok(Value::Float(number.to_f64().unwrap())),
            ArgumentPartKind::Int(number) => match number.to_i64() {
                Some(number) => Ok(Value::Int(number)),
                None => Err(ShellErrorKind::IntegerOverFlow),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum ArgumentPartKind {
    Variable(Variable),
    Expand(Expand),
    Bare(String),
    Float(BigDecimal),
    Int(BigInt),
    Quoted(String),
    Expr(Expr),
}

impl ArgumentPartKind {
    pub fn spanned(self, span: Span) -> ArgumentPart {
        ArgumentPart { kind: self, span }
    }
}

#[derive(Debug, Clone)]
pub struct Expand {
    pub content: Vec<ExpandKind>,
    pub span: Span,
}

impl Expand {
    pub fn eval(&self, ctx: &mut Context) -> Result<String, ShellErrorKind> {
        let mut value = String::new();
        for item in self.content.iter() {
            match item {
                ExpandKind::String(string) => value.push_str(string),
                ExpandKind::Expr(expr) => value.push_str(&expr.eval(ctx)?.try_into_string()?),
                ExpandKind::Variable(var) => value.push_str(&var.eval(ctx)?.try_into_string()?),
            }
        }
        Ok(value)
    }
}

#[derive(Debug, Clone)]
pub enum ExpandKind {
    String(String),
    Expr(Expr),
    Variable(Variable),
}

#[derive(Debug, Clone)]
pub struct Argument {
    pub parts: Vec<ArgumentPart>,
}

impl Argument {
    pub fn eval(&self, ctx: &mut Context) -> Result<Value, ShellErrorKind> {
        let mut parts = Vec::new();
        let mut glob = false;
        for part in self.parts.iter() {
            let (value, escape) = match &part.kind {
                ArgumentPartKind::Bare(value) => {
                    let mut string = value.to_string();
                    if string.contains('*') {
                        glob = true;
                    }
                    if string.starts_with('~') {
                        string = if glob {
                            string.replace(
                                '~',
                                &glob::Pattern::escape(&ctx.shell.home_dir().to_string_lossy()),
                            )
                        } else {
                            string.replace('~', &ctx.shell.home_dir().to_string_lossy())
                        }
                    }
                    (Value::from(string), false)
                }
                _ => (part.eval(ctx)?, true),
            };
            parts.push((escape, value));
        }

        if glob {
            let mut pattern = String::new();
            for (escape, value) in parts {
                pattern.push_str(&if escape {
                    glob::Pattern::escape(&value.unwrap_string())
                } else {
                    value.try_into_string()?
                });
            }

            let mut entries = Vec::new();
            for entry in glob::glob(&pattern)? {
                entries.push(Value::from(entry?.to_string_lossy().to_string()));
            }

            if !entries.is_empty() {
                Ok(Value::from(entries))
            } else {
                Err(ShellErrorKind::NoMatch(pattern))
            }
        } else {
            Ok(if parts.len() > 1 {
                let mut string = String::new();
                for (_, value) in parts {
                    string.push_str(&value.try_into_string()?);
                }
                Value::from(string)
            } else {
                parts.pop().unwrap().1
            })
        }
    }
}
