use std::{convert::TryInto, rc::Rc, sync::Arc};

use memchr::memchr;

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
use regex::Regex;
use syntax_error::{SyntaxError, SyntaxErrorKind};

use self::{
    ast::{
        expr::{argument::ArgumentPartKind, command::CommandPartKind, ExprKind},
        literal::{Literal, LiteralKind},
        statement::{function::Function, Statement},
    },
    lexer::token::{is_valid_identifier, span::Spanned},
    source::Source,
};

pub mod shell_error;
pub mod source;

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
        const NOTHING = 0;
        const INSIDE_LOOP = 0b00000001;
        const INSIDE_FUNCTION = 0b00000010;
    }
}

pub struct Parser {
    token: Option<Token>,
    lexer: Lexer,
}

impl Parser {
    pub fn new(name: String, src: String) -> Self {
        let mut lexer = Lexer::new(Source::new(name, src).into());
        let token = lexer.next();
        Self { lexer, token }
    }

    pub fn named_source(&self) -> Arc<Source> {
        self.lexer.named_source()
    }

    #[inline(always)]
    fn src(&self) -> &str {
        self.lexer.src()
    }

    #[inline(always)]
    fn get_span_from_src(&self, span: Span) -> &str {
        let start = span.start();
        let end = span.end();
        &self.src()[start..end]
    }

    /// Returns reference to first token that is not a comment
    #[inline(always)]
    fn peek(&mut self) -> Result<&Token> {
        loop {
            match self.token {
                Some(Token {
                    token_type: TokenType::Symbol(ref symbol),
                    ..
                }) if symbol == "#" => {
                    loop {
                        let token = self.eat()?;
                        if token.token_type == TokenType::NewLine {
                            break;
                        }
                    }
                    continue;
                }
                Some(ref token) => return Ok(token),
                None => return Err(SyntaxErrorKind::ExpectedToken),
            }
        }
    }

    #[inline(always)]
    fn eat(&mut self) -> Result<Token> {
        loop {
            let token = self.token.take();
            self.token = self.lexer.next();
            match token {
                Some(Token {
                    token_type: TokenType::Symbol(ref symbol),
                    ..
                }) if symbol == "#" => {
                    loop {
                        let token = self.eat()?;
                        if token.token_type == TokenType::NewLine {
                            break;
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
    fn peek_with_comment(&self) -> Result<&Token> {
        match self.token {
            Some(ref token) => Ok(token),
            None => Err(SyntaxErrorKind::ExpectedToken),
        }
    }

    #[inline(always)]
    fn eat_with_comment(&mut self) -> Result<Token> {
        let token = self.token.take();
        self.token = self.lexer.next();
        match token {
            Some(token) => Ok(token),
            None => Err(SyntaxErrorKind::ExpectedToken),
        }
    }

    #[inline(always)]
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
    pub fn skip_optional_space(&mut self) {
        while let Ok(token) = self.peek() {
            if !token.is_space() {
                break;
            }
            let _ = self.eat();
        }
    }

    #[inline(always)]
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

    pub fn parse(mut self) -> std::result::Result<Ast, P<SyntaxError>> {
        match self.parse_sequence(false, ParserContext::NOTHING) {
            Ok(sequence) => Ok(Ast::new(sequence, self.named_source())),
            Err(error) => Err(P::new(SyntaxError::new(
                error,
                self.src().to_string(),
                self.named_source().name.clone(),
            ))),
        }
    }

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

    fn parse_block(&mut self, ctx: ParserContext) -> Result<Block> {
        let start = self.eat()?.expect(TokenType::LeftBrace)?.span;
        let sequence = self.parse_sequence(true, ctx)?;
        let end = self.eat()?.expect(TokenType::RightBrace)?.span;
        Ok(Block {
            sequence,
            span: start + end,
        })
    }

    fn parse_compound(&mut self, ctx: ParserContext) -> Result<Compound> {
        let token_type = &self.peek()?.token_type;

        match token_type {
            TokenType::LeftBrace => {
                let block = self.parse_block(ctx)?;
                let span = block.span;
                Ok(StatementKind::Block(block).spanned(span).into())
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
                let name = match token.token_type {
                    TokenType::Symbol(name) => {
                        if !is_valid_identifier(&name) {
                            return Err(SyntaxErrorKind::InvalidIdentifier(token.span));
                        } else {
                            name
                        }
                    }
                    _ => return Err(SyntaxErrorKind::UnexpectedToken(token)),
                };

                self.skip_whitespace();
                self.eat()?.expect(TokenType::LeftParen)?;
                let mut vars: Vec<Variable> = Vec::new();
                loop {
                    self.skip_whitespace();
                    let token = self.eat()?;
                    match token.token_type {
                        TokenType::RightParen => break,
                        TokenType::Dollar => vars.push(self.parse_variable(false)?),
                        _ => return Err(SyntaxErrorKind::UnexpectedToken(token)),
                    }

                    self.skip_whitespace();
                    let token = self.eat()?;
                    match token.token_type {
                        TokenType::RightParen => break,
                        TokenType::Comma => (),
                        _ => return Err(SyntaxErrorKind::UnexpectedToken(token)),
                    }
                }
                self.skip_whitespace();
                let block = self.parse_block(ParserContext::INSIDE_FUNCTION)?;
                let end = block.span;

                let func = Function {
                    parameters: vars,
                    block,
                    src: self.named_source(),
                };

                Ok(StatementKind::Fn(name, Rc::new(func))
                    .spanned(start + end)
                    .into())
            }
            TokenType::Loop => {
                let start = self.eat()?.span;
                self.skip_whitespace();
                let block = self.parse_block(ctx | ParserContext::INSIDE_LOOP)?;
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
                let block = self.parse_block(ctx | ParserContext::INSIDE_LOOP)?;
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
                let block = self.parse_block(ctx | ParserContext::INSIDE_LOOP)?;
                let end = block.span;
                Ok(StatementKind::While(expr, block)
                    .spanned(start + end)
                    .into())
            }
            TokenType::If => Ok(self.parse_if(ctx)?.into()),
            TokenType::Break => {
                let span = self.eat()?.span;
                if !ctx.contains(ParserContext::INSIDE_LOOP) {
                    return Err(SyntaxErrorKind::BreakOutsideLoop(span));
                }
                Ok(StatementKind::Break.spanned(span).into())
            }
            TokenType::Continue => {
                let span = self.eat()?.span;
                if !ctx.contains(ParserContext::INSIDE_LOOP) {
                    return Err(SyntaxErrorKind::BreakOutsideLoop(span));
                }
                Ok(StatementKind::Continue.spanned(span).into())
            }
            TokenType::Return => {
                let start = self.eat()?.span;
                if !ctx.contains(ParserContext::INSIDE_FUNCTION) {
                    return Err(SyntaxErrorKind::ReturnOutsideFunction(start));
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

    fn parse_variable(&mut self, require_prefix: bool) -> Result<Variable> {
        if require_prefix {
            self.eat()?.expect(TokenType::Dollar)?;
        }

        if self.peek()?.token_type == TokenType::Dollar {
            self.eat()?.expect(TokenType::Dollar)?;
        }

        let token = self.eat()?;
        match token.token_type {
            TokenType::LeftBrace => {
                let var = Variable::try_from(self.eat()?)?;
                self.eat()?.expect(TokenType::RightBrace)?;
                Ok(var)
            }
            TokenType::QuestionMark if require_prefix => Ok(Variable {
                name: "?".into(),
                span: token.span,
            }),
            _ => Variable::try_from(token),
        }
    }

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

    fn parse_expand(&mut self) -> Result<Expand> {
        let mut span = self.eat()?.expect(TokenType::DoubleQuote)?.span;
        let mut content = Vec::new();

        // TODO add spans not just at the end
        let mut backslash = false;
        loop {
            let token = self.peek_with_comment()?;
            match token.token_type {
                TokenType::LeftParen if !backslash => {
                    content.push(ExpandKind::Expr(self.parse_sub_expr()?));
                }
                TokenType::Dollar if !backslash => {
                    content.push(ExpandKind::Variable(self.parse_variable(true)?));
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
                    let token = self.eat_with_comment()?;
                    let mut new = self.get_span_from_src(token.span).to_string();

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

                    match content.last_mut() {
                        Some(ExpandKind::String(string)) => string.push_str(&new),
                        _ => content.push(ExpandKind::String(new)),
                    }
                }
            }
        }

        Ok(Expand { content, span })
    }

    fn parse_if(&mut self, ctx: ParserContext) -> Result<Statement> {
        let start = self.eat()?.expect(TokenType::If)?.span;
        self.skip_optional_space();
        let expr = self.parse_expr(None, false)?;
        self.skip_optional_space();
        let block = self.parse_block(ctx)?;
        self.skip_optional_space();

        let statement = match self.peek() {
            Ok(token) => match token.token_type {
                TokenType::Else => {
                    self.eat()?;
                    self.skip_optional_space();
                    match self.peek()?.token_type {
                        TokenType::If => Some(P::new(self.parse_if(ctx)?)),
                        TokenType::LeftBrace => {
                            let block = self.parse_block(ctx)?;
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

    fn parse_regex_or_map(&mut self) -> Result<Expr> {
        self.eat()?.expect(TokenType::At)?;
        match self.peek()?.token_type {
            TokenType::LeftBrace => self.parse_map(),
            TokenType::Quote => self.parse_regex(),
            _ => Err(SyntaxErrorKind::UnexpectedToken(self.eat()?)),
        }
    }

    fn parse_map(&mut self) -> Result<Expr> {
        let mut span = self.eat()?.expect(TokenType::LeftBrace)?.span;
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

    fn parse_error_check(&mut self) -> Result<Expr> {
        let start = self.eat()?.expect(TokenType::QuestionMark)?.span;
        self.eat()?.expect(TokenType::LeftParen)?;
        self.skip_whitespace();
        let expr = self.parse_expr(None, true)?;
        self.skip_whitespace();
        let end = self.eat()?.expect(TokenType::RightParen)?.span;
        Ok(ExprKind::ErrorCheck(P::new(expr)).spanned(start + end))
    }

    fn parse_sub_expr(&mut self) -> Result<Expr> {
        let start = self.eat()?.expect(TokenType::LeftParen)?.span;
        self.skip_whitespace();
        let expr = self.parse_expr(None, true)?;
        self.skip_whitespace();
        let end = self.eat()?.expect(TokenType::RightParen)?.span;
        Ok(ExprKind::SubExpr(P::new(expr)).spanned(start + end))
    }

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

    fn parse_column(&mut self, expr: Expr) -> Result<Expr> {
        let start = self.eat()?.expect(TokenType::Dot)?.span;
        let token = self.eat()?;
        let column = match token.token_type {
            TokenType::Symbol(column) => column,
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

    fn parse_expr(&mut self, unop: Option<UnOp>, parse_cmd: bool) -> Result<Expr> {
        let primary = self.parse_primary(unop, parse_cmd)?;
        self.skip_optional_space();
        self.parse_expr_part(Some(primary), 0)
    }

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

    fn parse_pipe(&mut self, expr: Option<Expr>) -> Result<Expr> {
        let mut calls = match expr {
            Some(expr) => vec![expr],
            None => vec![self.parse_call()?],
        };
        let mut span = calls[0].span;

        self.skip_optional_space();
        while let Ok(token) = self.peek() {
            if token.token_type == TokenType::Pipe {
                self.eat()?;
                self.skip_whitespace();
                let expr = self.parse_call()?;
                span += expr.span;
                calls.push(expr);
            } else {
                break;
            }
        }
        Ok(ExprKind::Pipe(calls).spanned(span))
    }

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

    fn parse_command(&mut self) -> Result<Vec<CommandPart>> {
        if self.peek()?.token_type == TokenType::Exec {
            self.eat()?;
            self.skip_whitespace();
        }

        let mut parts = Vec::new();
        while let Ok(token) = self.peek() {
            let part = match &token.token_type {
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
                TokenType::Space
                | TokenType::NewLine
                | TokenType::SemiColon
                // by finding a left brace here a lambda function could be parsed
                | TokenType::LeftBrace
                | TokenType::RightBrace
                | TokenType::RightParen
                | TokenType::RightBracket
                | TokenType::Comma => break,
                _ => self.eat()?.try_into()?,
            };
            parts.push(part);
        }
        Ok(parts)
    }

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
            TokenType::QuestionMark => {
                let expr = self.parse_error_check()?;
                let span = expr.span;
                (ArgumentPartKind::Expr(expr).spanned(span), true)
            }
            // TODO list should maybe be parsed below to allow for concatination
            TokenType::LeftBracket => {
                let expr = self.parse_list()?;
                let span = expr.span;
                (ArgumentPartKind::Expr(expr).spanned(span), false)
            }
            // TODO same for map
            TokenType::At => {
                let expr = self.parse_regex_or_map()?;
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
                if !matches!(
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
