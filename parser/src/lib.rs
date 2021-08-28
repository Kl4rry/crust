use std::{convert::TryInto, rc::Rc};

pub mod lexer;

use lexer::{
    token::{Token, TokenType},
    Lexer,
};

pub mod ast;

use ast::{
    binop::BinOp, unop::UnOp, Argument, Ast, Block, Command, Compound, Direction, Expr, Identifier,
    Precedence, Statement, Variable,
};

pub mod error;
use error::{SyntaxError, SyntaxErrorKind};

pub type Result<T> = std::result::Result<T, SyntaxErrorKind>;
pub type P<T> = Box<T>;

pub struct Parser {
    token: Option<Token>,
    lexer: Lexer,
}

impl Parser {
    pub fn new(src: String) -> Self {
        let mut lexer = Lexer::new(src);
        let token = lexer.next();
        Self { lexer, token }
    }

    fn src(&self) -> Rc<String> {
        self.lexer.src()
    }

    #[inline(always)]
    fn token(&self) -> Result<&Token> {
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
        while let Ok(token) = self.token() {
            if !token.is_space() {
                return Ok(());
            }
            self.eat()?;
        }
        Err(SyntaxErrorKind::ExpectedToken)
    }

    #[inline(always)]
    pub fn skip_optional_space(&mut self) {
        while let Ok(token) = self.token() {
            if !token.is_space() {
                break;
            }
            let _ = self.eat();
        }
    }

    #[inline(always)]
    pub fn skip_whitespace(&mut self) {
        while let Ok(token) = self.token() {
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
        while let Ok(token) = self.token() {
            match token.token_type {
                TokenType::Space | TokenType::NewLine | TokenType::SemiColon => {
                    let _ = self.eat();
                }
                _ => break,
            }
        }
    }

    pub fn parse(&mut self) -> std::result::Result<Ast, SyntaxError> {
        match self.parse_sequence(false) {
            Ok(sequence) => Ok(Ast { sequence }),
            Err(error) => Err(SyntaxError::new(error, self.src())),
        }
    }

    fn parse_sequence(&mut self, block: bool) -> Result<Vec<Compound>> {
        let mut sequence = Vec::new();
        loop {
            let token = match self.token() {
                Ok(token) => token,
                Err(_) => return Ok(sequence),
            };
            match token.token_type {
                TokenType::Space | TokenType::NewLine | TokenType::SemiColon => drop(self.eat()),
                TokenType::RightBrace if block => return Ok(sequence),
                _ => sequence.push(self.parse_compound()?),
            };
        }
    }

    fn parse_block(&mut self) -> Result<Block> {
        self.eat()?.expect(TokenType::LeftBrace)?;
        let sequence = self.parse_sequence(true)?;
        self.skip_whitespace_and_semi();
        self.eat()?.expect(TokenType::RightBrace)?;
        Ok(Block { sequence })
    }

    fn parse_compound(&mut self) -> Result<Compound> {
        let token_type = &self.token()?.token_type;

        match token_type {
            TokenType::LeftBrace => Ok(Compound::Statement(Statement::Block(self.parse_block()?))),
            TokenType::Variable(_) => {
                let var: Variable = self.eat()?.try_into()?;
                while let Ok(token) = self.token() {
                    match token.token_type {
                        TokenType::Assignment => {
                            self.eat()?.expect(TokenType::Assignment)?;
                            self.skip_optional_space();
                            return Ok(Compound::Statement(Statement::Assignment(
                                var,
                                self.parse_expr(None)?,
                            )));
                        }
                        TokenType::Space => {
                            self.eat()?;
                        }
                        ref token_type => {
                            if token_type.is_binop() {
                                return Ok(Compound::Expr(self.parse_binop(Expr::Variable(var))?));
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
                Ok(Compound::Statement(Statement::Fn(name, vars, block)))
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
                match self.token() {
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
            TokenType::Symbol(symbol) => match symbol.as_str() {
                "export" => Ok(Compound::Statement(self.parse_declaration(true)?)),
                "let" => Ok(Compound::Statement(self.parse_declaration(false)?)),
                "alias" => Ok(Compound::Statement(self.parse_alias()?)),
                _ => Ok(Compound::Expr(self.parse_expr(None)?)),
            },
            TokenType::Exec
            | TokenType::Int(_, _)
            | TokenType::Float(_, _)
            | TokenType::String(_)
            | TokenType::ExpandString(_)
            | TokenType::Sub
            | TokenType::LeftParen
            | TokenType::Not => Ok(Compound::Expr(self.parse_expr(None)?)),
            _ => Err(SyntaxErrorKind::UnexpectedToken(self.eat().unwrap())),
        }
    }

    fn parse_if(&mut self) -> Result<Statement> {
        self.eat()?;
        self.skip_whitespace();
        let expr = self.parse_expr(None)?;
        self.skip_whitespace();
        let block = self.parse_block()?;
        self.skip_whitespace();

        let statement = match self.token() {
            Ok(token) => match token.token_type {
                TokenType::Else => {
                    self.eat()?;
                    self.skip_whitespace();
                    match self.token()?.token_type {
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

    fn parse_alias(&mut self) -> Result<Statement> {
        self.eat()?;
        self.skip_space()?;
        let arg = self.parse_argument()?;
        self.skip_optional_space();
        self.eat()?.expect(TokenType::Assignment)?;
        self.skip_optional_space();
        let expr = self.parse_expr(None)?;
        Ok(Statement::Alias(arg, expr))
    }

    fn parse_declaration(&mut self, export: bool) -> Result<Statement> {
        self.eat()?;
        self.skip_space()?;

        let token = self.eat()?;
        let variable: Variable = token.try_into()?;

        self.skip_optional_space();
        let token = match self.eat() {
            Ok(token) => token,
            Err(_) => {
                if export {
                    return Ok(Statement::Export(variable, None));
                } else {
                    return Ok(Statement::Declaration(variable, None));
                }
            }
        };

        match token.token_type {
            TokenType::Assignment => {
                let _ = self.skip_space();
                let expr = self.parse_expr(None)?;
                if export {
                    return Ok(Statement::Export(variable, Some(expr)));
                } else {
                    return Ok(Statement::Declaration(variable, Some(expr)));
                }
            }
            TokenType::SemiColon | TokenType::NewLine => {
                if export {
                    return Ok(Statement::Export(variable, None));
                } else {
                    return Ok(Statement::Declaration(variable, None));
                }
            }
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }

    fn parse_expr(&mut self, unop: Option<UnOp>) -> Result<Expr> {
        match &self.token()?.token_type {
            TokenType::True | TokenType::False => {
                let literal = Expr::Literal(self.eat()?.try_into()?);
                self.skip_optional_space();
                if let Ok(token) = self.token() {
                    if token.is_binop() {
                        return self.parse_binop(literal);
                    }
                }

                match unop {
                    Some(unop) => return Ok(Expr::Unary(unop, P::new(literal))),
                    None => return Ok(literal),
                }
            }
            TokenType::Symbol(_) => match unop {
                Some(unop) => return Ok(Expr::Unary(unop, P::new(self.parse_call()?))),
                None => return Ok(self.parse_call()?),
            },
            TokenType::Exec => Ok(self.parse_call()?),
            TokenType::LeftParen => {
                self.eat()?.expect(TokenType::LeftParen)?;
                self.skip_optional_space();
                let expr = self.parse_expr(None)?;
                self.skip_optional_space();
                self.eat()?.expect(TokenType::RightParen)?;

                match unop {
                    Some(unop) => Ok(Expr::Unary(unop, P::new(Expr::Paren(P::new(expr))))),
                    None => Ok(Expr::Paren(P::new(expr))),
                }
            }
            TokenType::Variable(_) => {
                let var = match unop {
                    Some(unop) => {
                        Expr::Unary(unop, P::new(Expr::Variable(self.eat()?.try_into()?)))
                    }
                    None => Expr::Variable(self.eat()?.try_into()?),
                };

                self.skip_optional_space();
                if let Ok(token) = self.token() {
                    if token.is_binop() {
                        return self.parse_binop(var);
                    }
                }
                // try for other Exprs here
                Ok(var)
            }
            TokenType::String(_)
            | TokenType::Int(_, _)
            | TokenType::Float(_, _)
            | TokenType::ExpandString(_) => {
                let literal = match unop {
                    Some(unop) => Expr::Unary(unop, P::new(Expr::Literal(self.eat()?.try_into()?))),
                    None => Expr::Literal(self.eat()?.try_into()?),
                };

                self.skip_optional_space();
                if let Ok(token) = self.token() {
                    if token.is_binop() {
                        return self.parse_binop(literal);
                    }
                }
                Ok(literal)
            }
            TokenType::Sub | TokenType::Not => {
                let inner = self.eat()?.try_into()?;
                match unop {
                    Some(outer) => Ok(Expr::Unary(outer, P::new(self.parse_expr(Some(inner))?))),
                    None => self.parse_expr(Some(inner)),
                }
            }
            _ => Err(SyntaxErrorKind::UnexpectedToken(self.eat().unwrap())),
        }
    }

    fn parse_binop(&mut self, mut lhs: Expr) -> Result<Expr> {
        let mut outer: BinOp = self.eat()?.try_into()?;
        self.skip_space()?;

        let mut rhs = self.parse_expr(None)?;

        match rhs {
            Expr::Binary(ref mut inner, ref mut rhs_l, ref mut rhs_r) => {
                if outer.precedence() > inner.precedence() {
                    // this madness corrects operator precedence
                    // this is an example of how to swaps correct the tree
                    //
                    // lhs = x
                    // rhs_l = z
                    // rhs_r = y
                    // outer = *
                    // inner = +
                    //
                    // x * z + y intital is parsed as below
                    //
                    //       *
                    //      / \
                    //     x   +
                    //        / \
                    //       z   y
                    std::mem::swap(&mut outer, inner);
                    // step 1 swap operators
                    //       +
                    //      / \
                    //     x   *
                    //        / \
                    //       z   y
                    std::mem::swap(&mut **rhs_r, &mut lhs);
                    // step 2 swap y and x
                    //       +
                    //      / \
                    //     y   *
                    //        / \
                    //       z   x
                    std::mem::swap(rhs_r, rhs_l);
                    // step 3 swap x and z
                    //       +
                    //      / \
                    //     y   *
                    //        / \
                    //       x   z
                    std::mem::swap(&mut rhs, &mut lhs);
                    // step 4 swap rhs and lhs
                    //       +
                    //      / \
                    //     *   y
                    //    / \
                    //   x   z
                }
            }
            _ => (),
        }

        Ok(Expr::Binary(outer, P::new(lhs), P::new(rhs)))
    }

    fn parse_call(&mut self) -> Result<Expr> {
        let command = self.parse_command()?;
        let mut args = Vec::new();

        while let Ok(token) = self.token() {
            match token.token_type {
                TokenType::Space => {
                    self.eat()?;
                }
                _ => {
                    if token.is_valid_id() {
                        args.push(self.parse_argument()?);
                    } else {
                        break;
                    }
                }
            }
        }
        let mut lhs = Expr::Call(command, args);

        self.skip_whitespace();

        if let Ok(token) = self.token() {
            match token.token_type {
                TokenType::Lt => {
                    self.eat()?;
                    self.skip_whitespace();
                    let rhs = self.parse_argument()?;
                    lhs = Expr::Redirect(Direction::Left, P::new(lhs), rhs);
                }
                TokenType::Gt => {
                    self.eat()?;
                    self.skip_whitespace();
                    let rhs = self.parse_argument()?;
                    lhs = Expr::Redirect(Direction::Right, P::new(lhs), rhs);
                }
                _ => (),
            }
        }

        self.skip_whitespace();

        if let Ok(token) = self.token() {
            match token.token_type {
                TokenType::Pipe => {
                    self.eat()?;
                    self.skip_whitespace();
                    let rhs = self.parse_call()?;
                    return Ok(Expr::Pipe(P::new(lhs), P::new(rhs)));
                }
                _ => (),
            }
        }

        Ok(lhs)
    }

    fn parse_command(&mut self) -> Result<Command> {
        let token = self.eat()?;
        match token.token_type {
            TokenType::Exec => {
                self.skip_optional_space();
                let token = self.eat()?;
                match token.token_type {
                    TokenType::ExpandString(text) => Ok(Command::Expand(text)),
                    TokenType::String(text) => Ok(Command::String(text)),
                    TokenType::Symbol(text) => Ok(Command::String(text)),
                    TokenType::Int(_, text) => Ok(Command::String(text)),
                    TokenType::Float(_, text) => Ok(Command::String(text)),
                    TokenType::Variable(_) => Ok(Command::Variable(token.try_into()?)),
                    _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
                }
            }
            TokenType::Symbol(text) => Ok(Command::String(text)),
            _ => Err(SyntaxErrorKind::UnexpectedToken(token)),
        }
    }

    fn parse_argument(&mut self) -> Result<Argument> {
        let mut ids = Vec::new();
        let id = self.eat()?.try_into_id()?;
        ids.push(id);
        while let Ok(token) = self.token() {
            if token.is_valid_id() {
                let token = self.eat()?;
                match token.token_type {
                    TokenType::String(string) => match ids.last_mut() {
                        Some(Identifier::String(text)) => {
                            text.push_str(&string);
                        }
                        _ => ids.push(Identifier::String(string.into())),
                    },
                    TokenType::Symbol(string) => match ids.last_mut() {
                        Some(Identifier::Glob(text)) => {
                            text.push_str(&string);
                        }
                        _ => ids.push(Identifier::Glob(string.into())),
                    },
                    TokenType::ExpandString(string) => match ids.last_mut() {
                        Some(Identifier::Expand(text)) => {
                            text.push_str(&string);
                        }
                        _ => ids.push(Identifier::Expand(string.into())),
                    },
                    TokenType::Variable(_) => ids.push(Identifier::Variable(token.try_into()?)),
                    TokenType::Int(_, string) => match ids.last_mut() {
                        Some(Identifier::Glob(text)) => {
                            text.push_str(&string);
                        }
                        _ => ids.push(Identifier::Glob(string.into())),
                    },
                    TokenType::Float(_, string) => match ids.last_mut() {
                        Some(Identifier::Glob(text)) => {
                            text.push_str(&string);
                        }
                        _ => ids.push(Identifier::Glob(string.into())),
                    },
                    _ => {
                        let string = token.try_into_glob_str()?;
                        match ids.last_mut() {
                            Some(Identifier::Glob(text)) => {
                                text.push_str(string);
                            }
                            _ => ids.push(Identifier::Glob(string.into())),
                        }
                    }
                }
            } else {
                return Ok(Argument { parts: ids });
            }
        }
        Ok(Argument { parts: ids })
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use super::*;

    #[test]
    fn parser_test() {
        let src = read_to_string("test.crust").unwrap();
        let mut parser = Parser::new(src);
        let ast = parser.parse();
        match &ast {
            Ok(ast) => println!("{:#?}", ast),
            Err(error) => eprintln!("{}", &error),
        }
        assert!(ast.is_ok());
    }
}
