use bigdecimal::{num_bigint::BigInt, BigDecimal, ToPrimitive};

use crate::{
    parser::{shell_error::ShellErrorKind, Expr, Variable, P},
    shell::{stream::OutputStream, value::Value},
    Shell,
};

#[derive(Debug, Clone)]
pub enum Argument {
    Variable(Variable),
    Expand(Expand),
    Bare(String),
    Float(BigDecimal),
    Int(BigInt),
    Quoted(String),
    Expr(P<Expr>),
}

impl Argument {
    pub fn eval(
        &self,
        shell: &mut Shell,
        output: &mut OutputStream,
    ) -> Result<Value, ShellErrorKind> {
        match self {
            Argument::Variable(var) => Ok(var.eval(shell)?),
            Argument::Expand(expand) => Ok(Value::String(expand.eval(shell, output)?)),
            Argument::Bare(string) => {
                let glob = string.contains('*');

                let value = if string.contains('~') {
                    let replacement = if glob {
                        shell.home_dir().to_string_lossy().to_string()
                    } else {
                        glob::Pattern::escape(&*shell.home_dir().to_string_lossy())
                    };
                    string.replace('~', &replacement)
                } else {
                    string.to_string()
                };

                if glob {
                    let mut entries = Vec::new();
                    for entry in glob::glob(&format!("./{}", &value))? {
                        entries.push(Value::String(entry?.to_string_lossy().to_string()));
                    }

                    if entries.is_empty() {
                        Err(ShellErrorKind::NoMatch(value))
                    } else {
                        Ok(Value::List(entries))
                    }
                } else {
                    Ok(Value::String(value))
                }
            },
            Argument::Quoted(string) => Ok(Value::String(string.clone())),
            Argument::Expr(expr) => Ok(expr.eval(shell, output)?),
            Argument::Float(number) => Ok(Value::Float(number.to_f64().unwrap())),
            Argument::Int(number) => match number.to_i128() {
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