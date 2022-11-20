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
    literal::Literal,
    statement::Statement,
    variable::Variable,
    Ast, Block, Compound, Direction, Precedence,
};

pub mod syntax_error;
use regex::Regex;
use syntax_error::{SyntaxError, SyntaxErrorKind};

use self::{ast::statement::function::Function, source::Source};

pub mod shell_error;
pub mod source;

pub type Result<T> = std::result::Result<T, SyntaxErrorKind>;

pub const ESCAPES: &[u8] = b"nt0rs\\";
pub const REPLACEMENTS: &[u8] = b"\n\t\0\r \\";

#[inline(always)]
pub fn escape_char(c: u8) -> u8 {
    memchr(c, ESCAPES).map(|i| REPLACEMENTS[i]).unwrap_or(c)
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
        match self.parse_sequence(false) {
            Ok(sequence) => Ok(Ast::new(sequence, self.named_source())),
            Err(error) => Err(P::new(SyntaxError::new(
                error,
                self.src().to_string(),
                self.named_source().name.clone(),
            ))),
        }
    }

    fn parse_sequence(&mut self, block: bool) -> Result<Vec<Compound>> {
        let mut sequence = Vec::new();

        loop {
            self.skip_whitespace_and_semi();
            let token = match self.peek() {
                Ok(token) => token,
                Err(_) => return Ok(sequence),
            };

            match token.token_type {
                TokenType::RightBrace if block => return Ok(sequence),
                _ => sequence.push(self.parse_compound()?),
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

    fn parse_block(&mut self) -> Result<Block> {
        self.eat()?.expect(TokenType::LeftBrace)?;
        let sequence = self.parse_sequence(true)?;
        self.eat()?.expect(TokenType::RightBrace)?;
        Ok(Block { sequence })
    }

    fn parse_compound(&mut self) -> Result<Compound> {
        let token_type = &self.peek()?.token_type;

        match token_type {
            TokenType::LeftBrace => Ok(Compound::Statement(Statement::Block(self.parse_block()?))),
            TokenType::Dollar => {
                let var: Variable = self.parse_variable(true)?;
                if let Ok(token) = self.peek() {
                    match token.token_type {
                        TokenType::Dot => {
                            return Ok(Compound::Expr(self.parse_column(Expr::Variable(var))?))
                        }
                        TokenType::LeftBracket => {
                            return Ok(Compound::Expr(self.parse_index(Expr::Variable(var))?))
                        }
                        _ => (),
                    }
                }

                while let Ok(token) = self.peek() {
                    match token.token_type {
                        TokenType::Assignment => {
                            self.eat()?.expect(TokenType::Assignment)?;
                            self.skip_optional_space();
                            return Ok(Compound::Statement(Statement::Assign(
                                var,
                                self.parse_expr(None)?,
                            )));
                        }
                        TokenType::Space => drop(self.eat()?),
                        ref token_type => {
                            if token_type.is_assign_op() {
                                let op = self.eat()?.to_assign_op();
                                self.skip_optional_space();
                                return Ok(Compound::Statement(Statement::AssignOp(
                                    var,
                                    op,
                                    self.parse_expr(None)?,
                                )));
                            } else if token_type.is_binop() {
                                return Ok(Compound::Expr(
                                    self.parse_expr_part(Some(Expr::Variable(var)), 0)?,
                                ));
                            } else {
                                return Err(SyntaxErrorKind::UnexpectedToken(self.eat()?));
                            }
                        }
                    }
                }
                Ok(Compound::Expr(Expr::Variable(var)))
            }
            TokenType::Fn => {
                self.eat()?;
                self.skip_whitespace();

                let token = self.eat()?;
                let name = match token.token_type {
                    TokenType::Symbol(name) => name,
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
                let block = self.parse_block()?;

                let func = Function {
                    parameters: vars,
                    block,
                    src: self.named_source(),
                };

                Ok(Compound::Statement(Statement::Fn(name, Rc::new(func))))
            }
            TokenType::Loop => {
                self.eat()?;
                self.skip_whitespace();
                let block = self.parse_block()?;
                Ok(Compound::Statement(Statement::Loop(block)))
            }
            TokenType::For => {
                self.eat()?;
                self.skip_whitespace();
                let var: Variable = self.parse_variable(false)?;
                self.skip_whitespace();
                self.eat()?.expect(TokenType::In)?;
                self.skip_whitespace();
                let expr = self.parse_expr(None)?;
                self.skip_whitespace();
                let block = self.parse_block()?;

                Ok(Compound::Statement(Statement::For(var, expr, block)))
            }
            TokenType::While => {
                self.eat()?;
                self.skip_whitespace();
                let expr = self.parse_expr(None)?;
                self.skip_whitespace();
                let block = self.parse_block()?;
                Ok(Compound::Statement(Statement::While(expr, block)))
            }
            TokenType::If => Ok(Compound::Statement(self.parse_if()?)),
            TokenType::Break => {
                self.eat()?;
                Ok(Compound::Statement(Statement::Break))
            }
            TokenType::Continue => {
                self.eat()?;
                Ok(Compound::Statement(Statement::Continue))
            }
            TokenType::Return => {
                self.eat()?;
                self.skip_optional_space();
                match self.peek() {
                    Ok(token) => match token.token_type {
                        TokenType::NewLine | TokenType::SemiColon => {
                            self.eat()?;
                            Ok(Compound::Statement(Statement::Return(None)))
                        }
                        _ => {
                            let expr = self.parse_expr(None)?;
                            Ok(Compound::Statement(Statement::Return(Some(expr))))
                        }
                    },
                    Err(_) => Ok(Compound::Statement(Statement::Return(None))),
                }
            }
            TokenType::Let => Ok(Compound::Statement(self.parse_declaration(false)?)),
            TokenType::Export => Ok(Compound::Statement(self.parse_declaration(true)?)),
            TokenType::Symbol(_) => Ok(Compound::Expr(self.parse_expr(None)?)),
            TokenType::Exec
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
            | TokenType::Not => Ok(Compound::Expr(self.parse_expr(None)?)),
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
        if token.token_type == TokenType::LeftBrace {
            let var = Variable::try_from(self.eat()?)?;
            self.eat()?.expect(TokenType::RightBrace)?;
            Ok(var)
        } else {
            Variable::try_from(token)
        }
    }

    fn parse_string(&mut self) -> Result<String> {
        self.eat()?.expect(TokenType::Quote)?;
        let mut string = String::new();

        loop {
            let token = self.eat_with_comment()?;
            match token.token_type {
                TokenType::Quote => break,
                _ => {
                    string.push_str(self.get_span_from_src(token.span));
                }
            }
        }
        Ok(string)
    }

    fn parse_expand(&mut self) -> Result<Expand> {
        self.eat()?.expect(TokenType::DoubleQuote)?;
        let mut expand = Expand {
            content: Vec::new(),
        };

        loop {
            let token = self.peek_with_comment()?;
            match token.token_type {
                TokenType::LeftParen => {
                    expand
                        .content
                        .push(ExpandKind::Expr(self.parse_sub_expr()?));
                }
                TokenType::Dollar => {
                    expand
                        .content
                        .push(ExpandKind::Variable(self.parse_variable(true)?));
                }
                TokenType::DoubleQuote => {
                    self.eat()?;
                    break;
                }
                _ => {
                    let token = self.eat_with_comment()?;
                    let new = self.get_span_from_src(token.span);
                    let mut escaped = String::new();
                    let mut index = 0;
                    while index < new.len() {
                        let byte = new.as_bytes()[index];
                        if byte == b'\\' {
                            index += 1;
                            let escape = escape_char(*new.as_bytes().get(index).unwrap_or(&b'\\'));
                            unsafe { escaped.as_mut_vec().push(escape) };
                        } else {
                            unsafe { escaped.as_mut_vec().push(byte) };
                        }
                        index += 1;
                    }
                    match expand.content.last_mut() {
                        Some(ExpandKind::String(string)) => string.push_str(&escaped),
                        _ => expand.content.push(ExpandKind::String(escaped)),
                    }
                }
            }
        }

        Ok(expand)
    }

    fn parse_if(&mut self) -> Result<Statement> {
        self.eat()?;
        self.skip_optional_space();
        let expr = self.parse_expr(None)?;
        self.skip_optional_space();
        let block = self.parse_block()?;
        self.skip_optional_space();

        let statement = match self.peek() {
            Ok(token) => match token.token_type {
                TokenType::Else => {
                    self.eat()?;
                    self.skip_optional_space();
                    match self.peek()?.token_type {
                        TokenType::If => Some(P::new(self.parse_if()?)),
                        TokenType::LeftBrace => Some(P::new(Statement::Block(self.parse_block()?))),
                        _ => return Err(SyntaxErrorKind::UnexpectedToken(self.eat()?)),
                    }
                }
                _ => None,
            },
            Err(_) => None,
        };

        Ok(Statement::If(expr, block, statement))
    }

    fn parse_declaration(&mut self, export: bool) -> Result<Statement> {
        self.eat()?.expect(TokenType::Let)?;
        self.skip_space()?;

        let variable: Variable = self.parse_variable(false)?;

        self.skip_optional_space();

        let token = self.eat()?;
        match token.token_type {
            TokenType::Assignment => {
                self.skip_optional_space();
                let expr = self.parse_expr(None)?;
                if export {
                    Ok(Statement::Export(variable, expr))
                } else {
                    Ok(Statement::Declaration(variable, expr))
                }
            }
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }

    fn parse_list(&mut self) -> Result<Expr> {
        self.eat()?.expect(TokenType::LeftBracket)?;
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
                        self.eat()?;
                        last_was_comma = true;
                    }
                }
                _ => {
                    last_was_comma = false;
                    list.push(self.parse_expr(None)?)
                }
            }
        }
        Ok(Expr::Literal(Literal::List(list)))
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
        self.eat()?.expect(TokenType::LeftBrace)?;
        let mut exprs: Vec<(Expr, Expr)> = Vec::new();
        let mut last_was_comma = true;
        loop {
            self.skip_whitespace();
            match self.peek()?.token_type {
                TokenType::RightBrace => {
                    self.eat()?;
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
                    let key = self.parse_expr(None)?;
                    self.skip_whitespace();
                    self.eat()?.expect(TokenType::Colon)?;
                    self.skip_whitespace();
                    let value = self.parse_expr(None)?;
                    exprs.push((key, value));
                }
            }
        }
        Ok(Expr::Literal(Literal::Map(exprs)))
    }

    fn parse_regex(&mut self) -> Result<Expr> {
        let token = self.peek()?; // TODO expect quote
        let start = token.span.start();
        let string = self.parse_string()?;

        let regex = match Regex::new(&string) {
            Ok(regex) => regex,
            Err(e) => {
                return Err(SyntaxErrorKind::Regex(
                    e,
                    Span::new(start, start - string.len()),
                ))
            }
        };

        Ok(Expr::Literal(Literal::Regex(Rc::new((regex, string)))))
    }

    fn parse_sub_expr(&mut self) -> Result<Expr> {
        self.eat()?.expect(TokenType::LeftParen)?;
        self.skip_whitespace();
        let expr = self.parse_expr(None)?;
        self.skip_whitespace();
        self.eat()?.expect(TokenType::RightParen)?;
        Ok(Expr::SubExpr(P::new(expr)))
    }

    fn parse_primary(&mut self, unop: Option<UnOp>) -> Result<Expr> {
        let expr = match self.peek()?.token_type {
            TokenType::True | TokenType::False => Expr::Literal(self.eat()?.try_into()?),
            TokenType::Symbol(_) | TokenType::Exec => self.parse_pipe(None)?,
            TokenType::LeftParen => self.parse_sub_expr()?.wrap(unop),
            TokenType::Dollar => Expr::Variable(self.parse_variable(true)?),
            TokenType::Quote => Expr::Literal(Literal::String(Rc::new(self.parse_string()?))),
            TokenType::Int(_, _) | TokenType::Float(_, _) => Expr::Literal(self.eat()?.try_into()?),
            TokenType::DoubleQuote => Expr::Literal(Literal::Expand(self.parse_expand()?)),
            TokenType::Sub | TokenType::Not => {
                let inner = self.eat()?.try_into()?;
                self.skip_optional_space();
                self.parse_expr(Some(inner))?
            }
            TokenType::LeftBracket => self.parse_list()?,
            TokenType::At => self.parse_regex_or_map()?,
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
        self.eat()?.expect(TokenType::Dot)?;
        let token = self.eat()?;
        let column = match token.token_type {
            TokenType::Symbol(column) => column,
            _ => return Err(SyntaxErrorKind::UnexpectedToken(token)),
        };
        let expr = Expr::Column(P::new(expr), column);

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
        self.eat()?.expect(TokenType::LeftBracket)?;
        self.skip_whitespace();
        let index = self.parse_expr(None)?;
        self.skip_whitespace();
        self.eat()?.expect(TokenType::RightBracket)?;
        let expr = Expr::Index {
            expr: P::new(expr),
            index: P::new(index),
        };

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

    fn parse_expr(&mut self, unop: Option<UnOp>) -> Result<Expr> {
        let primary = self.parse_primary(unop)?;
        self.skip_optional_space();
        self.parse_expr_part(Some(primary), 0)
    }

    fn parse_expr_part(&mut self, lhs: Option<Expr>, min_precedence: u8) -> Result<Expr> {
        let mut lhs = if let Some(expr) = lhs {
            expr
        } else {
            self.parse_primary(None)?
        };
        self.skip_optional_space();

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
            lhs = Expr::Binary(op, P::new(lhs), P::new(rhs));

            lookahead = match self.peek() {
                Ok(token) => {
                    if token.token_type == TokenType::Pipe {
                        return self.parse_pipe(Some(lhs));
                    }

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

        self.skip_optional_space();
        while let Ok(token) = self.peek() {
            if token.token_type == TokenType::Pipe {
                self.eat()?;
                self.skip_whitespace();
                calls.push(self.parse_call()?);
            } else {
                break;
            }
        }
        Ok(Expr::Pipe(calls))
    }

    fn parse_call(&mut self) -> Result<Expr> {
        let command = self.parse_command()?;
        let mut args = Vec::new();

        while let Ok(token) = self.peek() {
            match token.token_type {
                TokenType::Space => {
                    self.eat()?;
                }
                _ => {
                    if token.is_valid_argpart() {
                        args.push(self.parse_argument()?);
                    } else {
                        break;
                    }
                }
            }
        }
        Ok(Expr::Call(command, args))
    }

    fn parse_command(&mut self) -> Result<Vec<CommandPart>> {
        if self.peek()?.token_type == TokenType::Exec {
            self.eat()?;
            self.skip_whitespace();
        }

        let mut parts = Vec::new();
        while let Ok(token) = self.peek() {
            let part = match &token.token_type {
                TokenType::DoubleQuote => CommandPart::Expand(self.parse_expand()?),
                TokenType::Dollar => CommandPart::Variable(self.parse_variable(true)?),
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
            TokenType::Quote => (ArgumentPart::Quoted(self.parse_string()?), true),
            TokenType::DoubleQuote => (ArgumentPart::Expand(self.parse_expand()?), true),
            TokenType::LeftParen => (ArgumentPart::Expr(self.parse_sub_expr()?), true),
            // TODO list should maybe be parsed below to allow for concatination
            TokenType::LeftBracket => (ArgumentPart::Expr(self.parse_list()?), false),
            // TODO same for map
            TokenType::At => (ArgumentPart::Expr(self.parse_regex_or_map()?), false),
            TokenType::Dollar => (ArgumentPart::Variable(self.parse_variable(true)?), true),
            _ => (self.eat()?.try_into_argpart()?, true),
        };
        parts.push(part);

        if !concat {
            if let Ok(token) = self.peek() {
                if !matches!(
                    token.token_type,
                    TokenType::Space | TokenType::NewLine | TokenType::SemiColon
                ) {
                    return Err(SyntaxErrorKind::UnexpectedToken(self.eat()?));
                }
            }
        }

        while let Ok(token) = self.peek() {
            if token.is_valid_argpart() {
                match token.token_type {
                    TokenType::DoubleQuote => {
                        parts.push(ArgumentPart::Expand(self.parse_expand()?));
                        continue;
                    }
                    TokenType::LeftParen => {
                        parts.push(ArgumentPart::Expr(self.parse_sub_expr()?));
                        continue;
                    }
                    _ => (),
                }

                let token = self.eat()?;
                match token.token_type {
                    TokenType::Quote => {
                        let string = self.parse_string()?;
                        match parts.last_mut() {
                            Some(ArgumentPart::Quoted(text)) => {
                                text.push_str(&string);
                            }
                            _ => parts.push(ArgumentPart::Quoted(string)),
                        }
                    }
                    TokenType::Symbol(string) => match parts.last_mut() {
                        Some(ArgumentPart::Bare(text)) => {
                            text.push_str(&string);
                        }
                        _ => parts.push(ArgumentPart::Bare(string)),
                    },
                    TokenType::Dollar => {
                        parts.push(ArgumentPart::Variable(self.parse_variable(true)?))
                    }
                    TokenType::Int(number, _) => parts.push(ArgumentPart::Int(number.into())),
                    TokenType::Float(number, _) => parts.push(ArgumentPart::Float(number)),
                    _ => {
                        let string = token.try_into_glob_str()?;
                        match parts.last_mut() {
                            Some(ArgumentPart::Bare(text)) => {
                                text.push_str(string);
                            }
                            _ => parts.push(ArgumentPart::Bare(string.to_string())),
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
