use std::convert::TryFrom;

use super::context::Context;
use crate::{
    parser::{
        lexer::token::{is_valid_identifier, span::Span, Token, TokenType},
        shell_error::ShellErrorKind,
        syntax_error::SyntaxErrorKind,
    },
    shell::{builtins::variables, value::SpannedValue},
};

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub span: Span,
}

impl Variable {
    pub fn eval(&self, ctx: &mut Context) -> Result<SpannedValue, ShellErrorKind> {
        if let Some(value) = variables::get_var(ctx.shell, &self.name) {
            return Ok(value.spanned(self.span));
        }

        for frame in ctx.frame.clone() {
            if let Some(value) = frame.get_var(&self.name) {
                return Ok(value.spanned(self.span));
            }
        }
        Err(ShellErrorKind::VariableNotFound(self.name.clone()))
    }
}

impl TryFrom<Token> for Variable {
    type Error = SyntaxErrorKind;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        match token.token_type {
            TokenType::Symbol(name) => {
                if is_valid_identifier(&name) {
                    Err(SyntaxErrorKind::InvalidIdentifier(token.span))
                } else {
                    Ok(Self {
                        name,
                        span: token.span,
                    })
                }
            }
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }
}
