use std::convert::TryFrom;

use crate::{
    parser::{
        lexer::token::{Token, TokenType},
        shell_error::ShellError,
        syntax_error::SyntaxErrorKind,
    },
    shell::{builtins::variables, value::Value},
    Shell,
};

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
}

impl Variable {
    pub fn eval(&self, shell: &mut Shell) -> Result<Value, ShellError> {
        if let Some(value) = variables::get_var(shell, &self.name) {
            return Ok(value);
        }

        for frame in shell.stack.iter().rev() {
            if let Some(value) = frame.variables.get(&self.name) {
                return Ok(value.clone());
            }
        }

        match std::env::var(&self.name) {
            Ok(value) => Ok(Value::String(value)),
            Err(_) => Err(ShellError::VariableNotFound(self.name.clone())),
        }
    }
}

impl TryFrom<Token> for Variable {
    type Error = SyntaxErrorKind;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Variable(name) => Ok(Variable { name }),
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }
}
