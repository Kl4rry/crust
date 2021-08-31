use std::convert::TryInto;

use crate::{
    parser::{runtime_error::RunTimeError, Expr, Variable, P},
    shell::gc::Value,
    Shell,
};

#[derive(Debug)]
pub enum Identifier {
    Variable(Variable), // Should be expaned to variable value. Must be done before glob.
    Expand(Expand),     // Should be variable expanded.
    Bare(String),
    String(String),
    Expr(P<Expr>),
}

impl Identifier {
    pub fn eval(&mut self, shell: &mut Shell) -> Result<String, RunTimeError> {
        match self {
            Identifier::Variable(var) => Ok((*var.eval(shell)?).try_to_string()?),
            Identifier::Expand(_expand) => todo!(),
            Identifier::Bare(string) => Ok(string.clone()),
            Identifier::String(string) => Ok(string.clone()),
            Identifier::Expr(expr) => Ok(expr.eval(shell)?.try_to_string()?),
        }
    }
}

#[derive(Debug)]
pub struct Expand {
    pub content: Vec<ExpandKind>,
}

#[derive(Debug)]
pub enum ExpandKind {
    String(String),
    Expr(P<Expr>),
    Variable(Variable),
}

#[derive(Debug)]
pub struct Argument {
    pub parts: Vec<Identifier>,
}

impl Argument {
    pub fn eval(&mut self, shell: &mut Shell) -> Result<Vec<String>, RunTimeError> {
        let mut parts = Vec::new();
        let mut glob = false;
        for part in &mut self.parts {
            let (string, escape) = match part {
                Identifier::Bare(string) => {
                    if string.contains('*') {
                        glob = true;
                    }
                    (part.eval(shell).unwrap(), false)
                }
                _ => (part.eval(shell)?, true),
            };
            parts.push((escape, string));
        }

        if glob {
            let pattern: String = parts
                .into_iter()
                .map(|(escape, string)| {
                    if escape {
                        glob::Pattern::escape(&string)
                    } else {
                        string
                    }
                })
                .collect();
            let mut entries = Vec::new();
            for entry in glob::glob(&format!("./{}", &pattern))? {
                entries.push(entry?.to_string_lossy().to_string());
            }

            if entries.len() > 0 {
                Ok(entries)
            } else {
                Err(RunTimeError::NoMatchError)
            }
            
        } else {
            Ok(vec![parts.into_iter().map(|(_, string)| string).collect()])
        }
    }
}
