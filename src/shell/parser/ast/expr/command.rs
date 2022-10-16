use std::convert::{TryFrom, TryInto};

use crate::{
    parser::{
        ast::{expr::argument::Expand, Variable},
        lexer::token::{Token, TokenType},
        shell_error::ShellErrorKind,
        syntax_error::SyntaxErrorKind,
    },
    shell::{frame::Frame, stream::OutputStream},
    Shell,
};

#[derive(Debug, Clone)]
pub enum CommandPart {
    Expand(Expand),
    String(String),
    Variable(Variable),
}

impl CommandPart {
    pub fn eval(
        &self,
        shell: &mut Shell,
        frame: &mut Frame,
        output: &mut OutputStream,
    ) -> Result<String, ShellErrorKind> {
        match self {
            CommandPart::Variable(var) => Ok(var.eval(shell, frame)?.to_string()),
            CommandPart::Expand(expand) => Ok(expand.eval(shell, frame, output)?),
            CommandPart::String(string) => Ok(string.clone()),
        }
    }
}

impl TryFrom<Token> for CommandPart {
    type Error = SyntaxErrorKind;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::String(text) => Ok(CommandPart::String(text)),
            TokenType::Symbol(text) => Ok(CommandPart::String(text)),
            TokenType::Int(_, text) => Ok(CommandPart::String(text)),
            TokenType::Float(_, text) => Ok(CommandPart::String(text)),
            TokenType::Variable(_) => Ok(CommandPart::Variable(token.try_into()?)),
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }
}
