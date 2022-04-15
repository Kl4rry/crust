use std::convert::{TryFrom, TryInto};

use crate::{
    parser::{
        ast::{expr::argument::Expand, Variable},
        lexer::token::{Token, TokenType},
        shell_error::ShellErrorKind,
        syntax_error::SyntaxErrorKind,
    },
    shell::stream::OutputStream,
    Shell,
};

#[derive(Debug, Clone)]
pub enum Command {
    Expand(Expand),
    String(String),
    Variable(Variable),
}

impl Command {
    pub fn eval(
        &self,
        shell: &mut Shell,
        output: &mut OutputStream,
    ) -> Result<String, ShellErrorKind> {
        match self {
            Command::Variable(var) => Ok(var.eval(shell)?.as_ref().to_string()),
            Command::Expand(expand) => Ok(expand.eval(shell, output)?),
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
