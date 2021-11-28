use crate::{
    parser::{runtime_error::RunTimeError, Expr, Variable, P},
    Shell,
};

#[derive(Debug, Clone)]
pub enum Identifier {
    Variable(Variable), // Should be expaned to variable value. Must be done before glob.
    Expand(Expand),     // Should be variable expanded.
    Bare(String),
    String(String),
    Expr(P<Expr>),
}

impl Identifier {
    pub fn eval(&self, shell: &mut Shell) -> Result<String, RunTimeError> {
        match self {
            Identifier::Variable(var) => Ok(var.eval(shell)?.to_string()),
            Identifier::Expand(expand) => Ok(expand.eval(shell)?),
            Identifier::Bare(string) => Ok(string.clone()),
            Identifier::String(string) => Ok(string.clone()),
            Identifier::Expr(expr) => Ok(expr.eval(shell, false)?.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Expand {
    pub content: Vec<ExpandKind>,
}

impl Expand {
    pub fn eval(&self, shell: &mut Shell) -> Result<String, RunTimeError> {
        let mut value = String::new();
        for item in self.content.iter() {
            match item {
                ExpandKind::String(string) => value.push_str(string),
                ExpandKind::Expr(expr) => value.push_str(&expr.eval(shell, true)?.to_string()),
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
    pub fn eval(&self, shell: &mut Shell) -> Result<ArgumentValue, RunTimeError> {
        let mut parts = Vec::new();
        let mut glob = false;
        for part in self.parts.iter() {
            let (string, escape) = match part {
                Identifier::Bare(string) => {
                    let mut string = string.clone();
                    if string.contains('*') {
                        glob = true;
                    }
                    if string.contains('~') {
                        string = string.replace('~', shell.home_dir.as_os_str().to_str().unwrap());
                    }
                    (string, false)
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

            if !entries.is_empty() {
                Ok(ArgumentValue::Multi(entries))
            } else {
                Err(RunTimeError::NoMatch(pattern))
            }
        } else {
            Ok(ArgumentValue::Single(
                parts.into_iter().map(|(_, string)| string).collect(),
            ))
        }
    }
}

pub enum ArgumentValue {
    Single(String),
    Multi(Vec<String>),
}
