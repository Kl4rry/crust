use std::convert::TryFrom;

use crate::parser::{
    ast::{context::Context, expr::argument::Expand, Variable},
    lexer::token::{span::Span, Token, TokenType},
    shell_error::ShellErrorKind,
    syntax_error::SyntaxErrorKind,
};

#[derive(Debug, Clone)]
pub enum CommandPartKind {
    Expand(Expand),
    String(String),
    Variable(Variable),
}

impl CommandPartKind {
    pub fn spanned(self, span: Span) -> CommandPart {
        CommandPart { kind: self, span }
    }
}

#[derive(Debug, Clone)]
pub struct CommandPart {
    pub kind: CommandPartKind,
    pub span: Span,
}

impl CommandPart {
    pub fn eval(&self, ctx: &mut Context) -> Result<String, ShellErrorKind> {
        match &self.kind {
            CommandPartKind::Variable(var) => Ok(var.eval(ctx)?.to_string()),
            CommandPartKind::Expand(expand) => Ok(expand.eval(ctx)?),
            CommandPartKind::String(string) => Ok(string.clone()),
        }
    }
}

impl TryFrom<Token> for CommandPart {
    type Error = SyntaxErrorKind;
    fn try_from(token: Token) -> Result<Self, Self::Error> {
        let span = token.span;
        match token.token_type {
            TokenType::Div => Ok(CommandPartKind::String(String::from("/")).spanned(span)),
            TokenType::Dot => Ok(CommandPartKind::String(String::from(".")).spanned(span)),
            TokenType::Symbol(text) => Ok(CommandPartKind::String(text).spanned(span)),
            TokenType::Int(_, text) => Ok(CommandPartKind::String(text).spanned(span)),
            TokenType::Float(_, text) => Ok(CommandPartKind::String(text).spanned(span)),
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }
}
