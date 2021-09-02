use std::convert::{TryFrom, TryInto};

use crate::{
    parser::{
        ast::{expr::argument::Expand, Variable},
        lexer::token::{Token, TokenType},
        runtime_error::RunTimeError,
        syntax_error::SyntaxErrorKind,
    },
    Shell,
};

#[derive(Debug)]
pub enum Command {
    Expand(Expand),
    String(String),
    Variable(Variable),
}

impl Command {
    pub fn eval(&self, shell: &mut Shell) -> Result<String, RunTimeError> {
        match self {
            Command::Variable(var) => Ok((*var.eval(shell)?).try_to_string()?),
            Command::Expand(_expand) => todo!(),
            Command::String(string) => Ok(string.clone()),
        }
    }
}

impl TryFrom<Token> for Command {
    type Error = SyntaxErrorKind;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::String(text) => Ok(Command::String(text)),
            TokenType::Symbol(text) => Ok(Command::String(text)),
            TokenType::Int(_, text) => Ok(Command::String(text)),
            TokenType::Float(_, text) => Ok(Command::String(text)),
            TokenType::Variable(_) => Ok(Command::Variable(token.try_into()?)),
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }
}
