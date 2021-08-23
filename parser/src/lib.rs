use std::{collections::VecDeque, convert::TryInto};

pub mod lexer;
use lexer::{
    token::{Token, TokenType},
    Lexer,
};

pub mod ast;
use ast::{
    binop::BinOp, unop::UnOp, Argument, Ast, Command, Compound, Expr, Identifier, Precedence,
    Statement, Variable,
};

pub mod error;
use error::SyntaxError;

pub type Result<T> = std::result::Result<T, SyntaxError>;
pub type Small = smallstr::SmallString<[u8; 10]>;
pub type P<T> = Box<T>;

pub struct Parser {
    tokens: VecDeque<Token>,
}

impl Parser {
    pub fn new(src: String) -> Self {
        let lexer = Lexer::new(src);
        Self {
            tokens: lexer.collect(),
        }
    }

    #[inline(always)]
    fn token(&self) -> Result<&Token> {
        match self.tokens.front() {
            Some(token) => Ok(token),
            None => Err(SyntaxError::ExpectedToken),
        }
    }

    #[inline(always)]
    fn eat(&mut self) -> Result<Token> {
        match self.tokens.pop_front() {
            Some(token) => Ok(token),
            None => Err(SyntaxError::ExpectedToken),
        }
    }

    #[inline(always)]
    fn _peek(&self, offset: usize) -> Result<&Token> {
        if offset < self.tokens.len() {
            Ok(&self.tokens[offset])
        } else {
            Err(SyntaxError::ExpectedToken)
        }
    }

    pub fn skip_space(&mut self) -> Result<()> {
        while let Ok(token) = self.token() {
            if !token.is_space() {
                return Ok(());
            }
            self.eat()?;
        }
        Err(SyntaxError::ExpectedToken)
    }

    pub fn skip_optional_space(&mut self) {
        while let Ok(token) = self.token() {
            if !token.is_space() {
                break;
            }
            let _ = self.eat();
        }
    }

    pub fn parse(&mut self) -> Result<Ast> {
        Ok(Ast {
            sequence: self.parse_sequence()?,
        })
    }

    fn parse_sequence(&mut self) -> Result<Vec<Compound>> {
        let mut sequence = Vec::new();
        loop {
            let token = match self.token() {
                Ok(token) => token,
                Err(_) => return Ok(sequence),
            };
            match token.token_type {
                TokenType::Space | TokenType::NewLine | TokenType::SemiColon => drop(self.eat()),
                _ => sequence.push(self.parse_compound()?),
            };
        }
    }

    fn parse_compound(&mut self) -> Result<Compound> {
        let token_type = &self.token()?.token_type;

        match token_type {
            //parse ifs and loops and fn and other statements here
            TokenType::Variable(_) => {
                let var: Variable = self.eat()?.try_into()?;
                while let Ok(token) = self.eat() {
                    match token.token_type {
                        TokenType::Assignment => {
                            self.eat()?;
                            self.skip_space()?;
                            return Ok(Compound::Statement(Statement::Assignment(
                                var,
                                self.parse_expr(None)?,
                            )));
                        }
                        TokenType::Space => continue,
                        _ => (),
                    }
                }
                Ok(Compound::Expr(Expr::Variable(var)))
            }
            TokenType::Symbol(symbol) => match symbol.as_str() {
                "fn" => todo!("functions not implemented"),
                "loop" => todo!("loop not implemented"),
                "for" => todo!("for not implemented"),
                "while" => todo!("while not implemented"),
                "until" => todo!("until not implemented"),
                "if" => todo!("if not implemented"),
                "break" => {
                    self.eat()?;
                    self.skip_optional_space();
                    match self.eat() {
                        Ok(token) => match token.token_type {
                            TokenType::NewLine | TokenType::SemiColon => {
                                Ok(Compound::Statement(Statement::Break))
                            }
                            _ => Err(SyntaxError::UnexpectedToken(token)),
                        },
                        Err(_) => Ok(Compound::Statement(Statement::Break)),
                    }
                }
                "export" => todo!("export not implemented"),
                "let" => Ok(Compound::Statement(self.parse_declaration()?)),
                _ => Ok(Compound::Expr(self.parse_expr(None)?)),
            },
            TokenType::Exec
            | TokenType::String(_)
            | TokenType::ExpandString(_)
            | TokenType::Sub
            | TokenType::LeftParen
            | TokenType::Not => Ok(Compound::Expr(self.parse_expr(None)?)),
            _ => Err(SyntaxError::UnexpectedToken(self.eat().unwrap())),
        }
    }

    fn parse_declaration(&mut self) -> Result<Statement> {
        self.eat()?;
        self.skip_space()?;

        let token = self.eat()?;
        let variable: Variable = match token.token_type {
            TokenType::Variable(_) => token.try_into()?,
            _ => return Err(SyntaxError::UnexpectedToken(self.eat().unwrap())),
        };

        let _ = self.skip_space();
        let token = match self.eat() {
            Ok(token) => token,
            Err(_) => return Ok(Statement::Declaration(variable, None)),
        };

        match token.token_type {
            TokenType::Assignment => {
                let _ = self.skip_space();
                let expr = self.parse_expr(None)?;
                Ok(Statement::Declaration(variable, Some(expr)))
            }
            TokenType::SemiColon | TokenType::NewLine => Ok(Statement::Declaration(variable, None)),
            _ => Err(SyntaxError::UnexpectedToken(token)),
        }
    }

    fn parse_expr(&mut self, unop: Option<UnOp>) -> Result<Expr> {
        match &self.token()?.token_type {
            TokenType::Symbol(text) => {
                // function call parsing needs to happen here too
                // just look for starting parentheses

                match text.as_str() {
                    "true" | "false" => {
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
                    _ => (),
                };

                match unop {
                    Some(unop) => return Ok(Expr::Unary(unop, P::new(self.parse_call()?))),
                    None => return Ok(self.parse_call()?),
                }
            }
            TokenType::Exec => {
                self.eat()?;
                self.skip_space()?;
                Ok(self.parse_call()?)
            }
            TokenType::LeftParen => {
                self.eat()?;
                self.skip_optional_space();
                let expr = self.parse_expr(None)?;
                self.skip_optional_space();
                let right = self.eat()?.token_type;

                match right {
                    TokenType::RightParen => match unop {
                        Some(unop) => Ok(Expr::Unary(unop, P::new(Expr::Paren(P::new(expr))))),
                        None => Ok(Expr::Paren(P::new(expr))),
                    },
                    _ => Err(SyntaxError::ExpectedToken),
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
                let inner = self.parse_unop()?;
                match unop {
                    Some(outer ) => Ok(Expr::Unary(outer, P::new(self.parse_expr(Some(inner))?))),
                    None => self.parse_expr(Some(inner)),
                }
            }
            _ => Err(SyntaxError::UnexpectedToken(self.eat().unwrap())),
        }
    }

    // we need precedence rules and parentheses
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

    fn parse_unop(&mut self) -> Result<UnOp> {
        self.eat()?.try_into()
    }

    fn parse_call(&mut self) -> Result<Expr> {
        let command = self.parse_command()?;
        let mut args = Vec::new();

        while let Ok(token) = self.token() {
            match token.token_type {
                TokenType::Space => {
                    self.eat()?;
                }
                _ => match self.parse_argument() {
                    Ok(arg) => args.push(arg),
                    Err(_) => return Ok(Expr::Call(command, args)),
                },
            }
        }
        Ok(Expr::Call(command, args))
    }

    fn parse_command(&mut self) -> Result<Command> {
        let token = self.eat()?;
        match token.token_type {
            TokenType::ExpandString(text) => Ok(Command::Expand(text)),
            TokenType::String(text) => Ok(Command::String(text)),
            TokenType::Symbol(text) => Ok(Command::String(text)),
            TokenType::Variable(_) => Ok(Command::Variable(token.try_into()?)),
            _ => Err(SyntaxError::UnexpectedToken(token)),
        }
    }

    fn parse_argument(&mut self) -> Result<Argument> {
        let mut ids = Vec::new();
        let id = self.parse_identifier()?;
        ids.push(id);
        loop {
            match self.token() {
                Ok(_) => match self.parse_identifier() {
                    Ok(id) => ids.push(id),
                    Err(_) => return Ok(Argument { parts: ids }),
                },
                Err(_) => return Ok(Argument { parts: ids }),
            }
        }
    }

    fn parse_identifier(&mut self) -> Result<Identifier> {
        let token = self.eat().unwrap();
        match token.token_type {
            TokenType::String(text) => Ok(Identifier::Text(text.into())),
            TokenType::Symbol(text) => Ok(Identifier::Glob(text.into())),
            TokenType::ExpandString(text) => Ok(Identifier::Expand(text.into())),
            TokenType::Variable(_) => Ok(Identifier::Variable(token.try_into()?)),
            TokenType::Int(_, text) => Ok(Identifier::Text(text.into())),
            TokenType::Float(_, text) => Ok(Identifier::Text(text.into())),
            _ => Ok(Identifier::Glob(token.try_into_arg()?)),
        }
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
            Err(error) => eprintln!("{}", error),
        }
        assert!(ast.is_ok());
    }
}
