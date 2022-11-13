use std::convert::TryFrom;

use super::context::Context;
use crate::{
    parser::{
        lexer::token::{Token, TokenType},
        shell_error::ShellErrorKind,
        syntax_error::SyntaxErrorKind,
    },
    shell::{builtins::variables, value::Value},
};

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
}

impl Variable {
    pub fn eval(&self, ctx: &mut Context) -> Result<Value, ShellErrorKind> {
        if let Some(value) = variables::get_var(ctx.shell, &self.name) {
            return Ok(value);
        }

        for frame in ctx.frame.clone() {
            if let Some(value) = frame.get_var(&self.name) {
                return Ok(value);
            }
        }
        Err(ShellErrorKind::VariableNotFound(self.name.clone()))
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
