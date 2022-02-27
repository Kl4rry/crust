use bigdecimal::{num_bigint::BigInt, BigDecimal, ToPrimitive};

use crate::{
    parser::{shell_error::ShellErrorKind, Expr, Variable, P},
    shell::{stream::OutputStream, value::Value},
    Shell,
};

#[derive(Debug, Clone)]
pub enum Identifier {
    Variable(Variable),
    Expand(Expand),
    Bare(String),
    Float(BigDecimal),
    Int(BigInt),
    Quoted(String),
    Expr(P<Expr>),
}

impl Identifier {
    pub fn eval(
        &self,
        shell: &mut Shell,
        output: &mut OutputStream,
    ) -> Result<Value, ShellErrorKind> {
        match self {
            Identifier::Variable(var) => Ok(var.eval(shell)?),
            Identifier::Expand(expand) => Ok(Value::String(expand.eval(shell, output)?)),
            Identifier::Bare(value) => Ok(Value::String(value.to_string())),
            Identifier::Quoted(string) => Ok(Value::String(string.clone())),
            Identifier::Expr(expr) => Ok(expr.eval(shell, output)?),
            Identifier::Float(number) => Ok(Value::Float(number.to_f64().unwrap())),
            Identifier::Int(number) => match number.to_i128() {
                Some(number) => Ok(Value::Int(number)),
                None => Err(ShellErrorKind::IntegerOverFlow),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Expand {
    pub content: Vec<ExpandKind>,
}

impl Expand {
    pub fn eval(
        &self,
        shell: &mut Shell,
        output: &mut OutputStream,
    ) -> Result<String, ShellErrorKind> {
        let mut value = String::new();
        for item in self.content.iter() {
            match item {
                ExpandKind::String(string) => value.push_str(string),
                ExpandKind::Expr(expr) => value.push_str(&expr.eval(shell, output)?.to_string()),
                ExpandKind::Variable(var) => value.push_str(&var.eval(shell)?.to_string()),
            }
        }
        Ok(value)
    }
}

#[derive(Debug, Clone)]
pub enum ExpandKind {
    String(String),
    Expr(P<Expr>),
    Variable(Variable),
}

#[derive(Debug, Clone)]
pub struct Argument {
    pub parts: Vec<Identifier>,
}

impl Argument {
    pub fn eval(
        &self,
        shell: &mut Shell,
        output: &mut OutputStream,
    ) -> Result<ArgumentValue, ShellErrorKind> {
        let mut parts = Vec::new();
        let mut glob = false;
        for part in self.parts.iter() {
            let (string, escape) = match part {
                Identifier::Bare(value) => {
                    let mut string = value.to_string();
                    if string.contains('*') {
                        glob = true;
                    }
                    if string.contains('~') {
                        string = string.replace('~', &shell.home_dir().to_string_lossy());
                    }
                    (Value::String(string), false)
                }
                _ => (part.eval(shell, output)?, true),
            };
            parts.push((escape, string));
        }

        if glob {
            let pattern: String = parts
                .into_iter()
                .map(|(escape, string)| {
                    if escape {
                        glob::Pattern::escape(&string.unwrap_string())
                    } else {
                        // this should probably fail under some condition
                        // it does not make sense to try to use ever value stringyfied to glob
                        string.to_string()
                    }
                })
                .collect();
            let mut entries = Vec::new();
            for entry in glob::glob(&format!("./{}", &pattern))? {
                entries.push(Value::String(entry?.to_string_lossy().to_string()));
            }

            if !entries.is_empty() {
                Ok(ArgumentValue::Multi(entries))
            } else {
                Err(ShellErrorKind::NoMatch(pattern))
            }
        } else {
            Ok(ArgumentValue::Single(if parts.len() > 1 {
                // this should also fail under some conditions
                // and it should not alloacte a new string on every value
                Value::String(
                    parts
                        .into_iter()
                        .map(|(_, string)| string.to_string())
                        .collect(),
                )
            } else {
                parts.pop().unwrap().1
            }))
        }
    }
}

pub enum ArgumentValue {
    Single(Value),
    Multi(Vec<Value>),
}
