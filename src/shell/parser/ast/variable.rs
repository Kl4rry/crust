use std::convert::TryFrom;

use thin_string::ToThinString;

use crate::{
    parser::{
        lexer::token::{Token, TokenType},
        runtime_error::RunTimeError,
        syntax_error::SyntaxErrorKind,
    },
    shell::gc::{Value, ValueKind},
    Shell,
};

#[derive(Debug)]
pub struct Variable {
    pub name: String,
}

impl Variable {
    pub fn eval(&self, shell: &mut Shell) -> Result<ValueKind, RunTimeError> {
        let value = shell.variables.get(&self.name);
        match value {
            Some(value) => Ok(value.clone().into()),
            None => match std::env::var(&self.name) {
                Ok(value) => Ok(Value::String(value.to_thin_string()).into()),
                Err(_) => Err(RunTimeError::VariableNotFound),
            },
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