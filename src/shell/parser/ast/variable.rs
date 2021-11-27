use std::convert::TryFrom;

use thin_string::ToThinString;

use crate::{
    parser::{
        lexer::token::{Token, TokenType},
        runtime_error::RunTimeError,
        syntax_error::SyntaxErrorKind,
    },
    shell::values::{Value, ValueKind},
    Shell,
};

#[derive(Debug)]
pub struct Variable {
    pub name: String,
}

impl Variable {
    pub fn eval(&self, shell: &mut Shell) -> Result<ValueKind, RunTimeError> {
        for frame in shell.variable_stack.iter().rev() {
            if let Some(value) = frame.variables.get(&self.name) {
                return Ok(value.clone().into());
            }
        }

        match std::env::var(&self.name) {
            Ok(value) => Ok(Value::String(value.to_thin_string()).into()),
            Err(_) => Err(RunTimeError::VariableNotFound(self.name.clone())),
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
