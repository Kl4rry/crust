use std::{collections::VecDeque, convert::TryInto, rc::Rc, sync::Arc};

use memchr::memchr;
use miette::NamedSource;
use regex::Regex;
use tracing::{instrument, trace_span};

use crate::P;

pub mod lexer;

use lexer::{
    token::{span::Span, Token, TokenType},
    Lexer,
};

pub mod ast;

use ast::{
    expr::{
        argument::{Argument, ArgumentPart, Expand, ExpandKind},
        binop::BinOp,
        command::CommandPart,
        unop::UnOp,
        Expr,
    },
    statement::StatementKind,
    variable::Variable,
    Ast, Block, Compound, Direction, Precedence,
};

pub mod syntax_error;
use syntax_error::{SyntaxError, SyntaxErrorKind};

use self::{
    ast::{
        expr::{
            argument::ArgumentPartKind, closure::Closure, command::CommandPartKind, ExprKind,
            RedirectFd,
        },
        literal::{Literal, LiteralKind},
        statement::{function::Function, Statement},
    },
    lexer::token::{is_valid_identifier, span::Spanned},
};

pub mod shell_error;

pub type Result<T> = std::result::Result<T, SyntaxErrorKind>;

pub const ESCAPES: &[u8] = b"nt0rs\\";
pub const REPLACEMENTS: &[u8] = b"\n\t\0\r \\";

#[inline(always)]
pub fn escape_char(c: u8) -> Option<u8> {
    memchr(c, ESCAPES).map(|i| REPLACEMENTS[i])
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct ParserContext: u8 {
        const NOTHING = 1 << 0;
        const INSIDE_LOOP = 1 << 1;
        const INSIDE_FUNCTION = 1 << 2;
    }
}

#[derive(Debug)]
pub struct Parser {
    tokens: VecDeque<Token>,
    source: Arc<NamedSource<String>>,
    errors: Vec<SyntaxErrorKind>,
}

impl Parser {
    pub fn new(name: String, src: String) -> Self {
        let lexer = Lexer::new(NamedSource::new(name, src).into());
        let source = lexer.named_source();
        let span = trace_span!("lex");
        let _lex = span.enter();
        let tokens = lexer.collect();
        Self {
            source,
            tokens,
            errors: Vec::new(),
        }
    }

    pub fn named_source(&self) -> Arc<NamedSource<String>> {
        self.source.clone()
    }

    #[inline(always)]
    fn src(&self) -> &str {
        self.source.inner()
    }

    #[inline(always)]
    #[instrument(level = "trace")]
    fn get_span_from_src(&self, span: Span) -> &str {
        let start = span.start();
        let end = span.end();
        &self.src()[start..end]
    }

    /// Returns reference to first token that is not a comment
    #[inline(always)]
    #[instrument(level = "trace")]
    fn peek(&mut self) -> Result<&Token> {
        let mut i = 1;
        loop {
            let token = self.tokens.front();
            match token {
                Some(Token {
                    token_type: TokenType::Symbol(ref symbol),
                    ..
                }) if symbol == "#" => loop {
                    let token = self.tokens.get(i);
                    i += 1;
                    match token {
                        Some(token) => {
                            if token.token_type == TokenType::NewLine {
                                break;
                            }
                        }
                        None => return Err(SyntaxErrorKind::ExpectedToken),
                    }
                },
                Some(token) => return Ok(token),
                None => return Err(SyntaxErrorKind::ExpectedToken),
            }
        }
    }

    #[inline(always)]
    #[instrument(level = "trace")]
    fn eat(&mut self) -> Result<Token> {
        loop {
            let token = self.tokens.pop_front();
            match token {
                Some(Token {
                    token_type: TokenType::Symbol(ref symbol),
                    ..
                }) if symbol == "#" => {
                    loop {
                        let token = self.tokens.pop_front();
                        match token {
                            Some(token) => {
                                if token.token_type == TokenType::NewLine {
                                    break;
                                }
                            }
                            None => return Err(SyntaxErrorKind::ExpectedToken),
                        }
                    }
                    continue;
                }
                Some(token) => return Ok(token),
                None => return Err(SyntaxErrorKind::ExpectedToken),
            }
        }
    }

    #[inline(always)]
    #[instrument(level = "trace")]
    fn peek_with_comment(&self) -> Result<&Token> {
        match self.tokens.front() {
            Some(token) => Ok(token),
            None => Err(SyntaxErrorKind::ExpectedToken),
        }
    }

    #[inline(always)]
    #[instrument(level = "trace")]
    fn eat_with_comment(&mut self) -> Result<Token> {
        let token = self.tokens.pop_front();
        match token {
            Some(token) => Ok(token),
            None => Err(SyntaxErrorKind::ExpectedToken),
        }
    }

    #[inline(always)]
    #[instrument(level = "trace")]
    pub fn skip_space(&mut self) -> Result<()> {
        while let Ok(token) = self.peek() {
            if !token.is_space() {
                return Ok(());
            }
            self.eat()?;
        }
        Err(SyntaxErrorKind::ExpectedToken)
    }

    #[inline(always)]
    #[instrument(level = "trace")]
    pub fn skip_optional_space(&mut self) {
        while let Ok(token) = self.peek() {
            if !token.is_space() {
                break;
            }
            let _ = self.eat();
        }
    }

    #[inline(always)]
    #[instrument(level = "trace")]
    pub fn skip_whitespace(&mut self) {
        while let Ok(token) = self.peek() {
            match token.token_type {
                TokenType::Space | TokenType::NewLine => {
                    let _ = self.eat();
                }
                _ => break,
            }
        }
    }

    #[inline(always)]
    #[instrument(level = "trace")]
    pub fn skip_whitespace_and_semi(&mut self) {
        while let Ok(token) = self.peek() {
            match token.token_type {
                TokenType::Space | TokenType::NewLine | TokenType::SemiColon => {
                    let _ = self.eat();
                }
                _ => break,
            }
        }
    }

    #[instrument(level = "trace")]
    pub fn parse(mut self) -> (Option<Ast>, Vec<SyntaxError>) {
        match self.parse_sequence(false, ParserContext::NOTHING) {
            Ok(sequence) if self.errors.is_empty() => {
                (Some(Ast::new(sequence, self.named_source())), Vec::new())
            }
            Ok(sequence) => {
                let src = self.named_source();
                (
                    Some(Ast::new(sequence, src.clone())),
                    self.errors
                        .into_iter()
                        .map(|error| SyntaxError::new(error, src.clone()))
                        .collect(),
                )
            }
            Err(error) => {
                let src = self.named_source();
                let mut errors = vec![SyntaxError::new(error, src.clone())];
                errors.extend(
                    self.errors
                        .into_iter()
                        .map(|error| SyntaxError::new(error, src.clone())),
                );
                (None, errors)
            }
        }
    }

    #[instrument(level = "trace")]
    fn parse_sequence(&mut self, block: bool, ctx: ParserContext) -> Result<Vec<Compound>> {
        let mut sequence = Vec::new();

        loop {
            self.skip_whitespace_and_semi();
            let token = match self.peek() {
                Ok(token) => token,
                Err(_) => return Ok(sequence),
            };

            match token.token_type {
                TokenType::RightBrace if block => return Ok(sequence),
                _ => sequence.push(self.parse_compound(ctx)?),
            };
            self.skip_optional_space();

            if block {
                if let Ok(token) = self.peek() {
                    if token.token_type == TokenType::RightBrace {
                        return Ok(sequence);
                    }
                }
            }

            if let Ok(token) = self.eat() {
                match token.token_type {
                    TokenType::SemiColon | TokenType::NewLine => continue,
                    _ => return Err(SyntaxErrorKind::UnexpectedToken(token)),
                }
            }
        }
    }

    #[instrument(level = "trace")]
    fn parse_block(&mut self, ctx: ParserContext, opening_brace: Option<Token>) -> Result<Block> {
        let start = match opening_brace {
            Some(token) => token,
            None => self.eat()?,
        }
        .expect(TokenType::LeftBrace)?
        .span;
        let sequence = self.parse_sequence(true, ctx)?;
        let end = self.eat()?.expect(TokenType::RightBrace)?.span;
        Ok(Block {
            sequence,
            span: start + end,
        })
    }

    #[instrument(level = "trace")]
    fn parse_closure(&mut self, opening_brace: Option<Token>) -> Result<Expr> {
        let left_brace = match opening_brace {
            Some(token) => token,
            None => self.eat()?,
        }
        .expect(TokenType::LeftBrace)?;
        let start = left_brace.span;
        let mut vars: Vec<Variable> = Vec::new();
        let token = self.eat()?;
        let mut arg_span = token.span;
        match token.token_type {
            TokenType::Or => (),
            TokenType::Pipe => loop {
                self.skip_whitespace();
                let token = self.peek()?;
                match token.token_type {
                    TokenType::Pipe => {
                        self.eat()?;
                        break;
                    }
                    TokenType::Dollar | TokenType::Symbol(..) => {
                        vars.push(self.parse_variable(false)?);
                    }
                    _ => return Err(SyntaxErrorKind::UnexpectedToken(self.eat()?)),
                }

                self.skip_whitespace();
                let token = self.eat()?;
                arg_span += token.span;
                match token.token_type {
                    TokenType::Pipe => break,
                    TokenType::Comma => (),
                    _ => return Err(SyntaxErrorKind::UnexpectedToken(token)),
                }
            },
            _ => return Err(SyntaxErrorKind::UnexpectedToken(token)),
        }

        self.skip_whitespace();
        let block = self.parse_block(ParserContext::INSIDE_FUNCTION, Some(left_brace))?;
        let end = block.span;
        let span = start + end;
        Ok(ExprKind::Closure(
            Closure {
                span,
                arg_span,
                parameters: vars,
                block,
                src: self.named_source(),
            }
            .into(),
        )
        .spanned(span))
    }

    #[instrument(level = "trace")]
    fn parse_compound(&mut self, ctx: ParserContext) -> Result<Compound> {
        let token_type = &self.peek()?.token_type;

        match token_type {
            TokenType::LeftBrace => {
                let token = self.eat()?;
                self.skip_whitespace();
                if matches!(self.peek()?.token_type, TokenType::Pipe | TokenType::Or) {
                    Ok(self.parse_closure(Some(token))?.into())
                } else {
                    let block = self.parse_block(ctx, Some(token))?;
                    let span = block.span;
                    Ok(StatementKind::Block(block).spanned(span).into())
                }
            }
            TokenType::Dollar => {
                let var: Variable = self.parse_variable(true)?;
                let var_span = var.span;
                if let Ok(token) = self.peek() {
                    match token.token_type {
                        TokenType::Dot => {
                            return Ok(self
                                .parse_column(ExprKind::Variable(var).spanned(var_span))?
                                .into())
                        }
                        TokenType::LeftBracket => {
                            return Ok(self
                                .parse_index(ExprKind::Variable(var).spanned(var_span))?
                                .into())
                        }
                        _ => (),
                    }
                }

                while let Ok(token) = self.peek() {
                    match token.token_type {
                        TokenType::Assignment => {
                            self.eat()?.expect(TokenType::Assignment)?;
                            self.skip_optional_space();
                            let expr = self.parse_expr(None, false)?;
                            let expr_span = expr.span;
                            return Ok(StatementKind::Assign(var, expr)
                                .spanned(var_span + expr_span)
                                .into());
                        }
                        TokenType::Space => drop(self.eat()?),
                        TokenType::Pipe => {
                            return Ok(self
                                .parse_pipe(Some(Expr {
                                    kind: ExprKind::Variable(var),
                                    span: var_span,
                                }))?
                                .into());
                        }
                        ref token_type => {
                            if token_type.is_assign_op() {
                                let op = self.eat()?.to_assign_op();
                                self.skip_optional_space();
                                let expr = self.parse_expr(None, false)?;
                                let expr_span = expr.span;
                                return Ok(StatementKind::AssignOp(var, op, expr)
                                    .spanned(var_span + expr_span)
                                    .into());
                            } else if token_type.is_binop() {
                                return Ok(self
                                    .parse_expr_part(
                                        Some(ExprKind::Variable(var).spanned(var_span)),
                                        0,
                                    )?
                                    .into());
                            } else {
                                return Err(SyntaxErrorKind::UnexpectedToken(self.eat()?));
                            }
                        }
                    }
                }
                Ok(ExprKind::Variable(var).spanned(var_span).into())
            }
            TokenType::Fn => {
                let start = self.eat()?.span;
                self.skip_whitespace();

                let token = self.eat()?;
                let name: Rc<str> = match token.token_type {
                    TokenType::Symbol(name) => {
                        if !is_valid_identifier(&name) {
                            return Err(SyntaxErrorKind::InvalidIdentifier(token.span));
                        } else {
                            name.into()
                        }
                    }
                    _ => return Err(SyntaxErrorKind::UnexpectedToken(token)),
                };

                self.skip_whitespace();
                let mut arg_span = self.eat()?.expect(TokenType::LeftParen)?.span;
                let mut vars: Vec<Variable> = Vec::new();
                loop {
                    self.skip_whitespace();
                    let token = self.peek()?;
                    match token.token_type {
                        TokenType::RightParen => {
                            self.eat()?;
                            break;
                        }
                        TokenType::Dollar | TokenType::Symbol(..) => {
                            vars.push(self.parse_variable(false)?);
                        }
                        _ => return Err(SyntaxErrorKind::UnexpectedToken(self.eat()?)),
                    }

                    self.skip_whitespace();
                    let token = self.eat()?;
                    arg_span += token.span;
                    match token.token_type {
                        TokenType::RightParen => break,
                        TokenType::Comma => (),
                        _ => return Err(SyntaxErrorKind::UnexpectedToken(token)),
                    }
                }
                self.skip_whitespace();
                let block = self.parse_block(ParserContext::INSIDE_FUNCTION, None)?;
                let end = block.span;

                let func = Function {
                    name: name.clone(),
                    parameters: vars,
                    block,
                    arg_span,
                    src: self.named_source(),
                };

                Ok(StatementKind::Fn(name, Rc::new(func))
                    .spanned(start + end)
                    .into())
            }
            TokenType::Loop => {
                let start = self.eat()?.span;
                self.skip_whitespace();
                let block = self.parse_block(ctx | ParserContext::INSIDE_LOOP, None)?;
                let end = block.span;
                Ok(StatementKind::Loop(block).spanned(start + end).into())
            }
            TokenType::For => {
                let start = self.eat()?.span;
                self.skip_whitespace();
                let var: Variable = self.parse_variable(false)?;
                self.skip_whitespace();
                self.eat()?.expect(TokenType::In)?;
                self.skip_whitespace();
                let expr = self.parse_expr(None, false)?;
                self.skip_whitespace();
                let block = self.parse_block(ctx | ParserContext::INSIDE_LOOP, None)?;
                let end = block.span;

                Ok(StatementKind::For(var, expr, block)
                    .spanned(start + end)
                    .into())
            }
            TokenType::While => {
                let start = self.eat()?.span;
                self.skip_whitespace();
                let expr = self.parse_expr(None, false)?;
                self.skip_whitespace();
                let block = self.parse_block(ctx | ParserContext::INSIDE_LOOP, None)?;
                let end = block.span;
                Ok(StatementKind::While(expr, block)
                    .spanned(start + end)
                    .into())
            }
            TokenType::If => Ok(self.parse_if(ctx)?.into()),
            TokenType::Symbol(symbol) if symbol == "try" => {
                let span = self.eat()?.span;
                self.skip_whitespace();
                let block = self.parse_block(ctx, None)?;
                self.skip_whitespace();
                let token = self.eat()?;
                match &token.token_type {
                    TokenType::Symbol(symbol) if symbol == "catch" => (),
                    _ => return Err(SyntaxErrorKind::UnexpectedToken(token)),
                }
                self.skip_whitespace();
                let catch = self.parse_block(ctx, None)?;
                let span = span + catch.span;
                Ok(StatementKind::TryCatch(block, catch).spanned(span).into())
            }
            TokenType::Break => {
                let span = self.eat()?.span;
                if !ctx.contains(ParserContext::INSIDE_LOOP) {
                    self.errors.push(SyntaxErrorKind::BreakOutsideLoop(span));
                }
                Ok(StatementKind::Break.spanned(span).into())
            }
            TokenType::Continue => {
                let span = self.eat()?.span;
                if !ctx.contains(ParserContext::INSIDE_LOOP) {
                    self.errors.push(SyntaxErrorKind::ContinueOutsideLoop(span));
                }
                Ok(StatementKind::Continue.spanned(span).into())
            }
            TokenType::Return => {
                let start = self.eat()?.span;
                if !ctx.contains(ParserContext::INSIDE_FUNCTION) {
                    self.errors
                        .push(SyntaxErrorKind::ReturnOutsideFunction(start));
                }
                self.skip_optional_space();
                match self.peek() {
                    Ok(token) => match token.token_type {
                        TokenType::NewLine | TokenType::SemiColon => {
                            self.eat()?;
                            Ok(StatementKind::Return(None).spanned(start).into())
                        }
                        _ => {
                            let expr = self.parse_expr(None, false)?;
                            let end = expr.span;
                            Ok(StatementKind::Return(Some(expr))
                                .spanned(start + end)
                                .into())
                        }
                    },
                    Err(_) => Ok(StatementKind::Return(None).spanned(start).into()),
                }
            }
            TokenType::Let => Ok(self.parse_declaration(false)?.into()),
            TokenType::Export => Ok(self.parse_declaration(true)?.into()),
            TokenType::Symbol(_) => Ok(self.parse_expr(None, true)?.into()),
            TokenType::Exec
            | TokenType::QuestionMark
            | TokenType::Dot
            | TokenType::Div
            | TokenType::At
            | TokenType::Int(_, _)
            | TokenType::Float(_, _)
            | TokenType::Quote
            | TokenType::DoubleQuote
            | TokenType::Sub
            | TokenType::LeftParen
            | TokenType::True
            | TokenType::False
            | TokenType::LeftBracket
            | TokenType::Not => Ok(self.parse_expr(None, true)?.into()),
            _ => Err(SyntaxErrorKind::UnexpectedToken(self.eat()?)),
        }
    }

    #[instrument(level = "trace")]
    fn parse_variable(&mut self, require_prefix: bool) -> Result<Variable> {
        let mut has_prefix = false;
        if require_prefix || self.peek()?.token_type == TokenType::Dollar {
            self.eat()?.expect(TokenType::Dollar)?;
            has_prefix = true;
        }

        let token = self.eat()?;
        let start = {
            let mut start = token.span.start();
            if has_prefix {
                start -= 1;
            }
            Span::new(start, start + 1)
        };

        match token.token_type {
            TokenType::LeftBrace if require_prefix => {
                let token = self.eat()?;
                let mut var = match token.token_type {
                    TokenType::Gt => Variable {
                        name: ">".into(),
                        span: start + token.span,
                    },
                    TokenType::QuestionMark => Variable {
                        name: "?".into(),
                        span: start + token.span,
                    },
                    _ => Variable::try_from(token)?,
                };
                let end = self.eat()?.expect(TokenType::RightBrace)?.span;
                var.span = start + end;
                Ok(var)
            }
            TokenType::Gt if has_prefix => Ok(Variable {
                name: ">".into(),
                span: start + token.span,
            }),
            TokenType::QuestionMark if has_prefix => Ok(Variable {
                name: "?".into(),
                span: start + token.span,
            }),
            _ => Variable::try_from(token).map(|mut var| {
                var.span += start;
                var
            }),
        }
    }

    #[instrument(level = "trace")]
    fn parse_string(&mut self) -> Result<Spanned<String>> {
        let mut span = self.eat()?.expect(TokenType::Quote)?.span;
        let mut string = String::new();

        loop {
            let token = self.eat_with_comment()?;
            match token.token_type {
                TokenType::Quote => {
                    span += token.span;
                    break;
                }
                _ => {
                    string.push_str(self.get_span_from_src(token.span));
                }
            }
        }
        Ok(Spanned::new(string, span))
    }

    #[instrument(level = "trace")]
    fn parse_expand(&mut self) -> Result<Expand> {
        let mut span = self.eat()?.expect(TokenType::DoubleQuote)?.span;
        let mut content: Vec<Spanned<ExpandKind>> = Vec::new();

        // TODO add spans not just at the end
        let mut backslash = false;
        loop {
            let token = self.peek_with_comment()?;
            match token.token_type {
                TokenType::LeftParen if !backslash => {
                    let expr = self.parse_sub_expr()?;
                    let span = expr.span;
                    content.push(Spanned::new(ExpandKind::Expr(expr), span));
                }
                TokenType::Dollar if !backslash => {
                    let variable = self.parse_variable(true)?;
                    let span = variable.span;
                    content.push(Spanned::new(ExpandKind::Variable(variable), span));
                }
                TokenType::DoubleQuote if !backslash => {
                    span += self.eat()?.span;
                    break;
                }
                TokenType::Symbol(ref s) if s == "\\" => {
                    self.eat()?;
                    backslash = true;
                }
                _ => {
                    // TODO add list of spans for escapes ExpandKind::String(String, Vec<Span>)
                    let token = self.eat_with_comment()?;
                    let mut new = self.get_span_from_src(token.span).to_string();

                    let span = Span::new(
                        token.span.start().saturating_sub(backslash as usize),
                        token.span.end(),
                    );

                    if backslash {
                        let first = new.as_bytes()[0];
                        match escape_char(first) {
                            Some(escaped) => unsafe {
                                new.as_bytes_mut()[0] = escaped;
                            },
                            None => {
                                if new.as_bytes()[0] == b'x' {
                                    let bytes = new.as_bytes();
                                    if new.len() >= 3 {
                                        if (bytes[1] as char).is_ascii_hexdigit()
                                            && (bytes[2] as char).is_ascii_hexdigit()
                                        {
                                            let hex = u8::from_str_radix(&new[1..3], 16).unwrap();
                                            if hex > 127 {
                                                let mut span = token.span;
                                                span.set_len(3);
                                                return Err(SyntaxErrorKind::InvalidHexEscape(
                                                    span,
                                                ));
                                            }
                                            // safe because we know hex is an ascii char
                                            new.replace_range(0..3, unsafe {
                                                std::str::from_utf8_unchecked(&[hex])
                                            });
                                        } else {
                                            let mut span = token.span;
                                            span.set_len(3);
                                            return Err(SyntaxErrorKind::InvalidHexEscape(span));
                                        }
                                    } else {
                                        return Err(SyntaxErrorKind::InvalidHexEscape(token.span));
                                    }
                                } else {
                                    let chars = b"(\"$";
                                    if memchr(new.as_bytes()[0], chars).is_none() {
                                        new.insert(0, '\\')
                                    }
                                }
                            }
                        }
                        backslash = false;
                    }

                    let expr = content.last_mut().map(|s| {
                        s.span += span;
                        &mut s.inner
                    });
                    match expr {
                        Some(ExpandKind::String(string)) => string.push_str(&new),
                        _ => content.push(Spanned::new(ExpandKind::String(new), span)),
                    }
                }
            }
        }

        Ok(Expand { content, span })
    }

    #[instrument(level = "trace")]
    fn parse_if(&mut self, ctx: ParserContext) -> Result<Statement> {
        let start = self.eat()?.expect(TokenType::If)?.span;
        self.skip_optional_space();
        let expr = self.parse_expr(None, false)?;
        self.skip_optional_space();
        let block = self.parse_block(ctx, None)?;
        self.skip_optional_space();

        let statement = match self.peek() {
            Ok(token) => match token.token_type {
                TokenType::Else => {
                    self.eat()?;
                    self.skip_optional_space();
                    match self.peek()?.token_type {
                        TokenType::If => Some(P::new(self.parse_if(ctx)?)),
                        TokenType::LeftBrace => {
                            let block = self.parse_block(ctx, None)?;
                            let block_span = block.span;
                            Some(P::new(StatementKind::Block(block).spanned(block_span)))
                        }
                        _ => return Err(SyntaxErrorKind::UnexpectedToken(self.eat()?)),
                    }
                }
                _ => None,
            },
            Err(_) => None,
        };

        let end = match &statement {
            Some(statement) => statement.span,
            None => block.span,
        };

        Ok(StatementKind::If(expr, block, statement).spanned(start + end))
    }

    #[instrument(level = "trace")]
    fn parse_declaration(&mut self, export: bool) -> Result<Statement> {
        let start = self.eat()?.span;
        self.skip_space()?;

        let variable: Variable = self.parse_variable(false)?;

        self.skip_optional_space();

        let token = self.eat()?;
        match token.token_type {
            TokenType::Assignment => {
                self.skip_optional_space();
                let expr = self.parse_expr(None, false)?;
                let end = expr.span;
                if export {
                    Ok(StatementKind::Export(variable, expr).spanned(start + end))
                } else {
                    Ok(StatementKind::Declaration(variable, expr).spanned(start + end))
                }
            }
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }

    #[instrument(level = "trace")]
    fn parse_list(&mut self) -> Result<Expr> {
        let mut span = self.eat()?.expect(TokenType::LeftBracket)?.span;
        let mut list = Vec::new();
        let mut last_was_comma = true;
        loop {
            self.skip_whitespace();
            match self.peek()?.token_type {
                TokenType::RightBracket => {
                    self.eat()?;
                    break;
                }
                TokenType::Comma => {
                    if last_was_comma {
                        return Err(SyntaxErrorKind::UnexpectedToken(self.eat()?));
                    } else {
                        span += self.eat()?.span;
                        last_was_comma = true;
                    }
                }
                _ => {
                    last_was_comma = false;
                    let expr = self.parse_expr(None, false)?;
                    span += expr.span;
                    list.push(expr);
                }
            }
        }
        Ok(ExprKind::Literal(LiteralKind::List(list).spanned(span)).spanned(span))
    }

    #[instrument(level = "trace")]
    fn parse_regex_or_map(&mut self) -> Result<Expr> {
        self.eat()?.expect(TokenType::At)?;
        match self.peek()?.token_type {
            TokenType::LeftBrace => self.parse_map(None),
            TokenType::Quote => self.parse_regex(),
            _ => Err(SyntaxErrorKind::UnexpectedToken(self.eat()?)),
        }
    }

    #[instrument(level = "trace")]
    fn parse_map(&mut self, left_brace: Option<Token>) -> Result<Expr> {
        let left_brace = match left_brace {
            Some(token) => token,
            None => self.eat()?,
        }
        .expect(TokenType::LeftBrace)?;
        let mut span = left_brace.span;
        let mut exprs: Vec<(Expr, Expr)> = Vec::new();
        let mut last_was_comma = true;
        loop {
            self.skip_whitespace();
            match self.peek()?.token_type {
                TokenType::RightBrace => {
                    span += self.eat()?.span;
                    break;
                }
                TokenType::Comma => {
                    if last_was_comma {
                        return Err(SyntaxErrorKind::UnexpectedToken(self.eat()?));
                    } else {
                        self.eat()?;
                        last_was_comma = true;
                    }
                }
                _ => {
                    last_was_comma = false;
                    let key = self.parse_expr(None, false)?;
                    self.skip_whitespace();
                    self.eat()?.expect(TokenType::Colon)?;
                    self.skip_whitespace();
                    let value = self.parse_expr(None, false)?;
                    exprs.push((key, value));
                }
            }
        }
        Ok(ExprKind::Literal(LiteralKind::Map(exprs).spanned(span)).spanned(span))
    }

    #[instrument(level = "trace")]
    fn parse_regex(&mut self) -> Result<Expr> {
        let span = self.peek()?.span; // TODO expect quote
        let Spanned {
            inner: string,
            span: end,
        } = self.parse_string()?;
        let span = span + end;

        let regex = match Regex::new(&string) {
            Ok(regex) => regex,
            Err(e) => return Err(SyntaxErrorKind::Regex(e, span)),
        };

        Ok(
            ExprKind::Literal(LiteralKind::Regex(Rc::new((regex, string))).spanned(span))
                .spanned(span),
        )
    }

    #[instrument(level = "trace")]
    fn parse_bare_word(&mut self) -> Result<Expr> {
        let token = self.eat()?;
        match token.token_type {
            TokenType::Symbol(text) => Ok(ExprKind::Literal(
                LiteralKind::String(text.into()).spanned(token.span),
            )
            .spanned(token.span)),
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }

    #[instrument(level = "trace")]
    fn parse_error_check(&mut self) -> Result<Expr> {
        let start = self.eat()?.expect(TokenType::QuestionMark)?.span;
        self.eat()?.expect(TokenType::LeftParen)?;
        self.skip_whitespace();
        let expr = self.parse_expr(None, true)?;
        self.skip_whitespace();
        let end = self.eat()?.expect(TokenType::RightParen)?.span;
        Ok(ExprKind::ErrorCheck(P::new(expr)).spanned(start + end))
    }

    #[instrument(level = "trace")]
    fn parse_sub_expr(&mut self) -> Result<Expr> {
        let start = self.eat()?.expect(TokenType::LeftParen)?.span;
        self.skip_whitespace();
        let expr = self.parse_expr(None, true)?;
        self.skip_whitespace();
        let end = self.eat()?.expect(TokenType::RightParen)?.span;
        Ok(ExprKind::SubExpr(P::new(expr)).spanned(start + end))
    }

    #[instrument(level = "trace")]
    fn parse_primary(&mut self, unop: Option<UnOp>, parse_cmd: bool) -> Result<Expr> {
        let expr = match self.peek()?.token_type {
            TokenType::True | TokenType::False => {
                let token = self.eat()?;
                let span = token.span;
                ExprKind::Literal(token.try_into()?).spanned(span)
            }
            TokenType::Symbol(_) => {
                if parse_cmd {
                    self.parse_pipe(None)?
                } else {
                    self.parse_bare_word()?
                }
            }
            TokenType::Dot | TokenType::Div if parse_cmd => self.parse_pipe(None)?,
            TokenType::Exec => self.parse_pipe(None)?,
            TokenType::LeftParen => self.parse_sub_expr()?,
            TokenType::Dollar => {
                let var = self.parse_variable(true)?;
                let span = var.span;
                ExprKind::Variable(var).spanned(span)
            }
            TokenType::Quote => {
                let Spanned {
                    inner: string,
                    span,
                } = self.parse_string()?;
                ExprKind::Literal(LiteralKind::String(Rc::new(string)).spanned(span)).spanned(span)
            }
            TokenType::Int(_, _) | TokenType::Float(_, _) => {
                let literal: Literal = self.eat()?.try_into()?;
                let span = literal.span;
                ExprKind::Literal(literal).spanned(span)
            }
            TokenType::DoubleQuote => {
                let expand = self.parse_expand()?;
                let span = expand.span;
                ExprKind::Literal(LiteralKind::Expand(expand).spanned(span)).spanned(span)
            }
            TokenType::Sub | TokenType::Not => {
                let inner = self.eat()?.try_into()?;
                self.skip_optional_space();
                // TODO figure out if this should be parse_cmd or false
                self.parse_primary(Some(inner), parse_cmd)?
            }
            TokenType::LeftBracket => self.parse_list()?,
            TokenType::At => self.parse_regex_or_map()?,
            TokenType::QuestionMark => self.parse_error_check()?,
            TokenType::LeftBrace => {
                let token = self.eat()?;
                self.skip_whitespace();
                if matches!(self.peek()?.token_type, TokenType::Pipe | TokenType::Or) {
                    self.parse_closure(Some(token))?
                } else {
                    self.parse_map(Some(token))?
                }
            }
            _ => return Err(SyntaxErrorKind::UnexpectedToken(self.eat()?)),
        };

        match self.peek() {
            Ok(token) => {
                let expr = match token.token_type {
                    TokenType::Dot => self.parse_column(expr)?,
                    TokenType::LeftBracket => self.parse_index(expr)?,
                    _ => return Ok(expr.wrap(unop)),
                };
                Ok(expr.wrap(unop))
            }
            Err(_) => Ok(expr.wrap(unop)),
        }
    }

    #[instrument(level = "trace")]
    fn parse_column(&mut self, expr: Expr) -> Result<Expr> {
        let start = self.eat()?.expect(TokenType::Dot)?.span;
        let token = self.eat()?;
        let column = match token.token_type {
            TokenType::Symbol(column) => column,
            TokenType::Int(column, _) => column.to_string(),
            _ => return Err(SyntaxErrorKind::UnexpectedToken(token)),
        };
        let expr = ExprKind::Column(P::new(expr), column).spanned(start + token.span);

        match self.peek() {
            Ok(token) => {
                let expr = match token.token_type {
                    TokenType::Dot => self.parse_column(expr)?,
                    TokenType::LeftBracket => self.parse_index(expr)?,
                    _ => return Ok(expr),
                };
                Ok(expr)
            }
            Err(_) => Ok(expr),
        }
    }

    #[instrument(level = "trace")]
    fn parse_index(&mut self, expr: Expr) -> Result<Expr> {
        let left = self.eat()?.expect(TokenType::LeftBracket)?.span;
        self.skip_whitespace();
        let index = self.parse_expr(None, false)?;
        self.skip_whitespace();
        let right = self.eat()?.expect(TokenType::RightBracket)?.span;
        let expr = ExprKind::Index {
            expr: P::new(expr),
            index: P::new(index),
        }
        .spanned(left + right);

        match self.peek() {
            Ok(token) => {
                let expr = match token.token_type {
                    TokenType::Dot => self.parse_column(expr)?,
                    TokenType::LeftBracket => self.parse_index(expr)?,
                    _ => return Ok(expr),
                };
                Ok(expr)
            }
            Err(_) => Ok(expr),
        }
    }

    #[instrument(level = "trace")]
    fn parse_expr(&mut self, unop: Option<UnOp>, parse_cmd: bool) -> Result<Expr> {
        let primary = self.parse_primary(unop, parse_cmd)?;
        self.skip_optional_space();
        self.parse_expr_part(Some(primary), 0)
    }

    #[instrument(level = "trace")]
    fn parse_expr_part(&mut self, lhs: Option<Expr>, min_precedence: u8) -> Result<Expr> {
        let mut lhs = match lhs {
            Some(expr) => expr,
            None => self.parse_primary(None, false)?,
        };
        self.skip_optional_space();

        if let Ok(token) = self.peek() {
            if token.token_type == TokenType::Pipe {
                lhs = self.parse_pipe(Some(lhs))?;
            }
        }

        let mut lookahead: Option<BinOp> = match self.peek() {
            Ok(token) => {
                if token.is_binop() {
                    Some(token.to_binop())
                } else {
                    None
                }
            }
            Err(_) => None,
        };

        while let Some(op) = lookahead {
            let (precedence, assoc) = op.precedence();
            if precedence < min_precedence {
                break;
            }

            let next_min = if assoc == Direction::Left {
                precedence + 1
            } else {
                precedence
            };

            self.eat()?;
            self.skip_whitespace();
            let rhs = self.parse_expr_part(None, next_min)?;
            self.skip_optional_space();

            if op.kind.is_comparison() {
                if let ExprKind::Binary(lhs_op, _, _) = lhs.kind {
                    if lhs_op.is_comparison() {
                        return Err(SyntaxErrorKind::ComparisonChaining(op.span, lhs_op.span));
                    }
                }

                if let ExprKind::Binary(rhs_op, _, _) = rhs.kind {
                    if rhs_op.is_comparison() {
                        return Err(SyntaxErrorKind::ComparisonChaining(op.span, rhs_op.span));
                    }
                }
            }

            let lhs_span = lhs.span;
            let rhs_span = rhs.span;
            lhs = ExprKind::Binary(op, P::new(lhs), P::new(rhs)).spanned(lhs_span + rhs_span);

            lookahead = match self.peek() {
                Ok(token) => {
                    if token.is_binop() {
                        Some(token.to_binop())
                    } else {
                        None
                    }
                }
                Err(_) => None,
            };
        }
        self.skip_optional_space();
        Ok(lhs)
    }

    #[instrument(level = "trace")]
    fn parse_pipe(&mut self, expr: Option<Expr>) -> Result<Expr> {
        let mut calls = match expr {
            Some(expr) => vec![expr],
            None => vec![self.parse_call()?],
        };
        let mut span = calls[0].span;

        self.skip_optional_space();
        while let Ok(token) = self.peek() {
            match token.token_type {
                TokenType::Pipe => {
                    self.eat()?;
                    self.skip_whitespace();
                    let expr = self.parse_call()?;
                    span += expr.span;
                    calls.push(expr);
                }
                TokenType::Gt => {
                    let redirect = self.parse_redirect()?;
                    span += redirect.span;
                    calls.push(redirect);

                    loop {
                        self.skip_optional_space();
                        if let Ok(token) = self.peek() {
                            match token.token_type {
                                TokenType::Gt => {
                                    let redirect = self.parse_redirect()?;
                                    span += redirect.span;
                                    calls.push(redirect);
                                }
                                _ => break,
                            }
                        } else {
                            break;
                        }
                    }
                }
                _ => {
                    break;
                }
            }
        }
        Ok(ExprKind::Pipe(calls).spanned(span))
    }

    fn parse_redirect(&mut self) -> Result<Expr> {
        self.eat()?;
        let mut append = false;
        if let Ok(token) = self.peek() {
            if token.token_type == TokenType::Gt {
                self.eat()?;
                append = true;
            }
        }

        self.skip_whitespace();
        let arg = self.parse_argument()?;
        let arg_span = arg.span();
        Ok(ExprKind::Redirection {
            arg,
            append,
            fd: RedirectFd::Stdout,
        }
        .spanned(arg_span))
    }

    #[instrument(level = "trace")]
    fn parse_call(&mut self) -> Result<Expr> {
        let command = self.parse_command()?;
        let mut span = command[0].span;
        let mut args = Vec::new();

        while let Ok(token) = self.peek() {
            match token.token_type {
                TokenType::Space => {
                    self.eat()?;
                }
                _ => {
                    if token.is_valid_argpart() {
                        let arg = self.parse_argument()?;
                        span += arg.parts.last().unwrap().span;
                        args.push(arg);
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(ExprKind::Call(command, args).spanned(span))
    }

    #[instrument(level = "trace")]
    fn parse_command(&mut self) -> Result<Vec<CommandPart>> {
        if self.peek()?.token_type == TokenType::Exec {
            self.eat()?;
            self.skip_whitespace();
        }

        self.peek()?;

        let mut parts = Vec::new();
        while let Ok(token) = self.peek() {
            let part = match &token.token_type {
                TokenType::Control => return Err(SyntaxErrorKind::UnexpectedToken(self.eat()?)),
                TokenType::DoubleQuote => {
                    let expand = self.parse_expand()?;
                    let span = expand.span;
                    CommandPartKind::Expand(expand).spanned(span)
                },
                TokenType::Dollar => {
                    let var = self.parse_variable(true)?;
                    let span = var.span;
                    CommandPartKind::Variable(var).spanned(span)
                },
                // TODO this check should be possible to remove and just have the is_valid_argpart
                TokenType::Space
                | TokenType::NewLine
                | TokenType::SemiColon
                // by finding a left brace here a lambda function could be parsed
                | TokenType::LeftBrace
                | TokenType::RightBrace
                | TokenType::RightParen
                | TokenType::RightBracket
                | TokenType::Comma => break,
                _ => {
                    if token.token_type.is_valid_argpart() {
                        self.eat()?.try_into()?
                    } else  {
                        break
                    }
                },
            };
            parts.push(part);
        }

        Ok(parts)
    }

    #[instrument(level = "trace")]
    fn parse_argument(&mut self) -> Result<Argument> {
        let mut parts = Vec::new();

        let (part, concat) = match self.peek()?.token_type {
            TokenType::Quote => {
                let Spanned { inner, span } = self.parse_string()?;
                (ArgumentPartKind::Quoted(inner).spanned(span), true)
            }
            TokenType::DoubleQuote => {
                let expand = self.parse_expand()?;
                let span = expand.span;
                (ArgumentPartKind::Expand(expand).spanned(span), true)
            }
            TokenType::LeftParen => {
                let expr = self.parse_sub_expr()?;
                let span = expr.span;
                (ArgumentPartKind::Expr(expr).spanned(span), true)
            }
            TokenType::LeftBracket => {
                let expr = self.parse_list()?;
                let span = expr.span;
                (ArgumentPartKind::Expr(expr).spanned(span), false)
            }
            TokenType::At => {
                let expr = self.parse_regex_or_map()?;
                let span = expr.span;
                (ArgumentPartKind::Expr(expr).spanned(span), false)
            }
            TokenType::LeftBrace => {
                let expr = self.parse_closure(None)?;
                let span = expr.span;
                (ArgumentPartKind::Expr(expr).spanned(span), false)
            }
            TokenType::Dollar => {
                let var = self.parse_variable(true)?;
                let span = var.span;
                (ArgumentPartKind::Variable(var).spanned(span), true)
            }
            _ => (self.eat()?.try_into_argpart()?, true),
        };
        parts.push(part);

        if !concat {
            if let Ok(token) = self.peek() {
                if token.token_type == TokenType::RightParen {
                    return Ok(Argument { parts });
                } else if !matches!(
                    token.token_type,
                    TokenType::Space | TokenType::NewLine | TokenType::SemiColon | TokenType::Pipe
                ) {
                    return Err(SyntaxErrorKind::UnexpectedToken(self.eat()?));
                }
            }
        }

        while let Ok(token) = self.peek() {
            if token.is_valid_argpart() {
                match token.token_type {
                    TokenType::DoubleQuote => {
                        let expand = self.parse_expand()?;
                        let span = expand.span;
                        parts.push(ArgumentPartKind::Expand(expand).spanned(span));
                        continue;
                    }
                    TokenType::LeftParen => {
                        let expr = self.parse_sub_expr()?;
                        let span = expr.span;
                        parts.push(ArgumentPartKind::Expr(expr).spanned(span));
                        continue;
                    }
                    _ => (),
                }

                let token = self.peek()?;
                match token.token_type {
                    TokenType::Quote => {
                        let string = self.parse_string()?;
                        match parts.last_mut() {
                            Some(ArgumentPart {
                                kind: ArgumentPartKind::Quoted(text),
                                span,
                            }) => {
                                text.push_str(&string.inner);
                                *span += string.span;
                            }
                            _ => parts
                                .push(ArgumentPartKind::Quoted(string.inner).spanned(string.span)),
                        }
                    }
                    TokenType::Dollar => {
                        let var = self.parse_variable(true)?;
                        let span = var.span;
                        parts.push(ArgumentPartKind::Variable(var).spanned(span))
                    }
                    _ => {
                        let token = self.eat()?;
                        match token.token_type {
                            TokenType::Symbol(string) => match parts.last_mut() {
                                Some(ArgumentPart {
                                    kind: ArgumentPartKind::Bare(text),
                                    span,
                                }) => {
                                    *span += token.span;
                                    text.push_str(&string);
                                }
                                _ => parts.push(ArgumentPartKind::Bare(string).spanned(token.span)),
                            },
                            TokenType::Int(number, _) => {
                                parts.push(ArgumentPartKind::Int(number.into()).spanned(token.span))
                            }
                            TokenType::Float(number, _) => {
                                parts.push(ArgumentPartKind::Float(number).spanned(token.span))
                            }
                            _ => {
                                let string_span = token.span;
                                let string = token.try_into_glob_str()?;
                                match parts.last_mut() {
                                    Some(ArgumentPart {
                                        kind: ArgumentPartKind::Bare(text),
                                        span,
                                    }) => {
                                        text.push_str(string);
                                        *span += string_span;
                                    }
                                    _ => parts.push(
                                        ArgumentPartKind::Bare(string.to_string())
                                            .spanned(string_span),
                                    ),
                                }
                            }
                        }
                    }
                }
            } else {
                break;
            }
        }

        Ok(Argument { parts })
    }
}
