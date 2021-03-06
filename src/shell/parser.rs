use std::{convert::TryInto, rc::Rc};

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

use self::lexer::escape_char;

pub mod shell_error;

pub type Result<T> = std::result::Result<T, SyntaxErrorKind>;

pub struct Parser {
    token: Option<Token>,
    lexer: Lexer,
    name: String,
}

impl Parser {
    pub fn new(src: String, name: String) -> Self {
        let mut lexer = Lexer::new(src);
        let token = lexer.next();
        Self { lexer, token, name }
    }

    fn src(&self) -> &str {
        self.lexer.src()
    }

    #[inline(always)]
    fn get_span_from_src(&self, span: Span) -> &str {
        let start = span.start();
        let end = span.end();
        &self.src()[start..end]
    }

    #[inline(always)]
    fn peek(&self) -> Result<&Token> {
        match self.token {
            Some(ref token) => Ok(token),
            None => Err(SyntaxErrorKind::ExpectedToken),
        }
    }

    #[inline(always)]
    fn eat(&mut self) -> Result<Token> {
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

    pub fn parse(mut self) -> std::result::Result<Ast, SyntaxError> {
        match self.parse_sequence(false) {
            Ok(sequence) => Ok(Ast::new(
                sequence,
                self.src().to_string(),
                self.name.clone(),
            )),
            Err(error) => Err(SyntaxError::new(
                error,
                self.src().to_string(),
                self.name.clone(),
            )),
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
            TokenType::Variable(_) => {
                let var: Variable = self.eat()?.try_into()?;
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
                        TokenType::Variable(_) => vars.push(token.try_into()?),
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
                Ok(Compound::Statement(Statement::Fn(
                    name,
                    Rc::new((vars, block)),
                )))
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
                let var: Variable = self.eat()?.try_into()?;
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
            | TokenType::Dollar
            | TokenType::Int(_, _)
            | TokenType::Float(_, _)
            | TokenType::String(_)
            | TokenType::Quote
            | TokenType::Sub
            | TokenType::LeftParen
            | TokenType::True
            | TokenType::False
            | TokenType::LeftBracket
            | TokenType::Not => Ok(Compound::Expr(self.parse_expr(None)?)),
            _ => Err(SyntaxErrorKind::UnexpectedToken(self.eat()?)),
        }
    }

    fn parse_expand(&mut self) -> Result<Expand> {
        self.eat()?.expect(TokenType::Quote)?;
        let mut expand = Expand {
            content: Vec::new(),
        };

        loop {
            let token = self.peek()?;
            match token.token_type {
                TokenType::LeftParen => {
                    expand
                        .content
                        .push(ExpandKind::Expr(self.parse_sub_expr()?));
                }
                TokenType::Variable(_) => {
                    expand
                        .content
                        .push(ExpandKind::Variable(self.eat()?.try_into()?));
                }
                TokenType::Quote => {
                    self.eat()?;
                    break;
                }
                _ => {
                    let token = self.eat()?;
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
        self.eat()?;
        self.skip_space()?;

        let variable: Variable = self.eat()?.try_into()?;

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
            TokenType::String(_) => self.parse_regex(),
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
        let token = self.eat()?;
        let string = match token.token_type {
            TokenType::String(string) => string,
            _ => return Err(SyntaxErrorKind::UnexpectedToken(token)),
        };

        let regex = match Regex::new(&string) {
            Ok(regex) => regex,
            Err(e) => return Err(SyntaxErrorKind::Regex(e, token.span)),
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
            TokenType::Variable(_) => Expr::Variable(self.eat()?.try_into()?),
            TokenType::String(_) | TokenType::Int(_, _) | TokenType::Float(_, _) => {
                Expr::Literal(self.eat()?.try_into()?)
            }
            TokenType::Quote => Expr::Literal(Literal::Expand(self.parse_expand()?)),
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
                TokenType::Quote => CommandPart::Expand(self.parse_expand()?),
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
            TokenType::Quote => (ArgumentPart::Expand(self.parse_expand()?), true),
            TokenType::LeftParen => (ArgumentPart::Expr(self.parse_sub_expr()?), true),
            // todo list should maybe be parsed below to allow for concatination
            TokenType::LeftBracket => (ArgumentPart::Expr(self.parse_list()?), false),
            // todo same for map
            TokenType::At => (ArgumentPart::Expr(self.parse_regex_or_map()?), false),
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
                    TokenType::Quote => {
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
                    TokenType::String(string) => match parts.last_mut() {
                        Some(ArgumentPart::Quoted(text)) => {
                            text.push_str(&string);
                        }
                        _ => parts.push(ArgumentPart::Quoted(string)),
                    },
                    TokenType::Symbol(string) => match parts.last_mut() {
                        Some(ArgumentPart::Bare(text)) => {
                            text.push_str(&string);
                        }
                        _ => parts.push(ArgumentPart::Bare(string)),
                    },
                    TokenType::Variable(_) => parts.push(ArgumentPart::Variable(token.try_into()?)),
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

        // todo
        // fix this fucking thing
        // it should make it a real expression and eval it with the correct operands
        // or it could just accept that it is a string
        let last = parts.last().unwrap();
        if let ArgumentPart::Int(_) = last {
            if let ArgumentPart::Bare(s) = parts.first().unwrap() {
                if s.bytes().all(|b| b == b'-') {
                    let neg = !s.len() % 2 == 0;
                    let last = parts.pop().unwrap();
                    let mut number = match last {
                        ArgumentPart::Int(number) => number,
                        _ => unreachable!(),
                    };
                    parts.clear();
                    if neg {
                        number = -number;
                    }
                    parts.push(ArgumentPart::Int(number));
                }
            }
        } else if let ArgumentPart::Float(_) = last {
            if let ArgumentPart::Bare(s) = parts.first().unwrap() {
                if s.bytes().all(|b| b == b'-') {
                    let neg = !s.len() % 2 == 0;
                    let last = parts.pop().unwrap();
                    let mut number = match last {
                        ArgumentPart::Float(number) => number,
                        _ => unreachable!(),
                    };
                    parts.clear();
                    if neg {
                        number = -number;
                    }
                    parts.push(ArgumentPart::Float(number));
                }
            }
        }

        Ok(Argument { parts })
    }
}
