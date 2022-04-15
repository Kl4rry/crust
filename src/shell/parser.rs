use std::{convert::TryInto, rc::Rc};

pub mod lexer;

use lexer::{
    token::{span::Span, Token, TokenType},
    Lexer,
};

pub mod ast;

use ast::{
    expr::{
        argument::{Argument, Expand, ExpandKind},
        binop::BinOp,
        command::Command,
        unop::UnOp,
        Expr,
    },
    literal::Literal,
    statement::Statement,
    variable::Variable,
    Ast, Block, Compound, Direction, Precedence,
};

pub mod syntax_error;
use syntax_error::{SyntaxError, SyntaxErrorKind};

use self::lexer::escape_char;

pub mod shell_error;

pub type Result<T> = std::result::Result<T, SyntaxErrorKind>;
pub type P<T> = Box<T>;

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
        self.eat()?.expect(TokenType::Space)?;
        while let Ok(token) = self.peek() {
            if !token.is_space() {
                return Ok(());
            }
            self.eat()?;
        }
        Ok(())
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
                                    self.parse_sub_expr(Some(Expr::Variable(var)), 0)?,
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

    fn parse_expr_expand(&mut self) -> Result<Expr> {
        self.eat()?.expect(TokenType::Dollar)?;
        let expr = Expr::SubExpr(P::new(self.parse_expr(None)?));
        Ok(expr)
    }

    fn parse_expand(&mut self) -> Result<Expand> {
        self.eat()?.expect(TokenType::Quote)?;
        let mut expand = Expand {
            content: Vec::new(),
        };

        loop {
            let token = self.peek()?;
            match token.token_type {
                TokenType::Dollar => expand
                    .content
                    .push(ExpandKind::Expr(P::new(self.parse_expr_expand()?))),
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
        self.eat()?;
        let mut list = Vec::new();
        let mut last_was_comma = false;
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

    fn parse_primary(&mut self, unop: Option<UnOp>) -> Result<Expr> {
        match self.peek()?.token_type {
            TokenType::True | TokenType::False => {
                Ok(Expr::Literal(self.eat()?.try_into()?).wrap(unop))
            }
            TokenType::Symbol(_) | TokenType::Exec => Ok(self.parse_pipe()?.wrap(unop)),
            TokenType::LeftParen => {
                self.eat()?.expect(TokenType::LeftParen)?;
                self.skip_whitespace();
                let expr = self.parse_expr(None)?;
                self.skip_whitespace();
                self.eat()?.expect(TokenType::RightParen)?;
                Ok(Expr::Paren(P::new(expr)).wrap(unop))
            }
            TokenType::Variable(_) => Ok(Expr::Variable(self.eat()?.try_into()?).wrap(unop)),
            TokenType::Dollar => Ok(self.parse_expr_expand()?.wrap(unop)),
            TokenType::String(_)
            | TokenType::Int(_, _)
            | TokenType::Float(_, _)
            | TokenType::Quote => Ok(Expr::Literal(match self.peek()?.token_type {
                TokenType::Quote => {
                    let expand = self.parse_expand()?;
                    Literal::Expand(expand)
                }
                _ => self.eat()?.try_into()?,
            })
            .wrap(unop)),
            TokenType::Sub | TokenType::Not => {
                let inner = self.eat()?.try_into()?;
                self.skip_optional_space();
                Ok(self.parse_expr(Some(inner))?.wrap(unop))
            }
            TokenType::LeftBracket => self.parse_list(),
            _ => Err(SyntaxErrorKind::UnexpectedToken(self.eat()?)),
        }
    }

    fn parse_expr(&mut self, unop: Option<UnOp>) -> Result<Expr> {
        let primary = self.parse_primary(unop)?;
        self.skip_optional_space();
        self.parse_sub_expr(Some(primary), 0)
    }

    fn parse_sub_expr(&mut self, lhs: Option<Expr>, min_precedence: u8) -> Result<Expr> {
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
            let rhs = self.parse_sub_expr(None, next_min)?;
            self.skip_optional_space();

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

            lhs = Expr::Binary(op, P::new(lhs), P::new(rhs));
        }
        self.skip_optional_space();
        Ok(lhs)
    }

    fn parse_pipe(&mut self) -> Result<Expr> {
        let mut calls = vec![self.parse_call()?];

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

        loop {
            if self.peek().is_ok() {
                self.skip_space()?;
            } else {
                break;
            }

            if let Ok(token) = self.peek() {
                if token.is_valid_arg() {
                    args.push(self.parse_argument()?);
                } else {
                    break;
                }
            }
        }

        Ok(Expr::Call(command, args))
    }

    fn parse_command(&mut self) -> Result<Command> {
        if self.peek()?.token_type == TokenType::Exec {
            self.eat()?;
            self.skip_whitespace();
        }

        let token = self.peek()?;
        match &token.token_type {
            TokenType::Quote => Ok(Command::Expand(self.parse_expand()?)),
            _ => Command::try_from(self.eat()?),
        }
    }

    fn parse_argument(&mut self) -> Result<Argument> {
        match self.peek()?.token_type {
            TokenType::Quote => Ok(Argument::Expand(self.parse_expand()?)),
            TokenType::Dollar => Ok(Argument::Expr(P::new(self.parse_expr_expand()?))),
            TokenType::LeftBracket => Ok(Argument::Expr(P::new(self.parse_list()?))),
            _ => {
                let mut arg = self.eat()?.try_into_arg()?;
                if let Argument::Bare(ref mut string) = arg {
                    while let Ok(token) = self.peek() {
                        if token.is_valid_arg() {
                            let token = self.eat()?;
                            let text = match token.token_type {
                                TokenType::Symbol(text) => text,
                                _ => token.try_into_glob_str()?.to_string(),
                            };
                            string.push_str(&text);
                        } else {
                            break;
                        }
                    }
                }
                Ok(arg)
            }
        }
    }
}
