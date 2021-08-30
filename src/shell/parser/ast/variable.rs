use std::{convert::TryFrom, rc::Rc};

use thin_string::ToThinString;

use crate::{
    parser::{
        lexer::token::{Token, TokenType},
        runtime_error::RunTimeError,
        syntax_error::SyntaxErrorKind,
    },
    shell::gc::Value,
    Shell,
};

#[derive(Debug)]
pub struct Variable {
    pub name: String,
}

impl Variable {
    pub fn eval(&self, shell: &mut Shell) -> Result<Rc<Value>, RunTimeError> {
        let value = shell.variables.get(&self.name);
        match value {
            Some(value) => Ok(value.clone()),
            None => match std::env::var(&self.name) {
                Ok(value) => Ok(Rc::new(Value::String(value.to_thin_string()))),
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
