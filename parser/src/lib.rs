use std::{collections::VecDeque, convert::TryInto};

mod lexer;
use lexer::{
    token::{Token, TokenType},
    Lexer,
};

mod ast;
use ast::{Argument, Ast, Command, Compound, Expr, Identifier, Statement, Variable};

mod error;
use error::SyntaxError;

type Result<T> = std::result::Result<T, SyntaxError>;

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
        return Err(SyntaxError::ExpectedToken);
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
                TokenType::Exec
                | TokenType::Variable(_)
                | TokenType::Symbol(_)
                | TokenType::ExpandString(_)
                | TokenType::String(_) => sequence.push(self.parse_compound()?),
                TokenType::Space | TokenType::NewLine => drop(self.eat()),
                _ => return Err(SyntaxError::UnexpectedToken(self.eat().unwrap())),
            };
        }
    }

    fn parse_compound(&mut self) -> Result<Compound> {
        let token_type = &self.token().unwrap().token_type;

        match token_type {
            //parse ifs and loops and fn and other statements here
            TokenType::Variable(_) => {
                let var: Variable = self.eat().unwrap().try_into()?;
                while let Ok(token) = self.eat() {
                    match token.token_type {
                        TokenType::Assignment => {
                            self.eat()?;
                            self.skip_space()?;
                            return Ok(Compound::Statement(Statement::Assignment(
                                var,
                                self.parse_expr()?,
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
                "if" => todo!("if not implemented"),
                "let" => Ok(Compound::Statement(self.parse_declaration()?)),
                _ => Ok(Compound::Expr(self.parse_expr()?)),
            },
            TokenType::Exec | TokenType::String(_) | TokenType::ExpandString(_) => {
                Ok(Compound::Expr(self.parse_expr()?))
            }
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
                let expr = self.parse_expr()?;
                return Ok(Statement::Declaration(variable, Some(expr)));
            }
            TokenType::SemiColon | TokenType::NewLine => {
                return Ok(Statement::Declaration(variable, None))
            }
            _ => return Err(SyntaxError::UnexpectedToken(token)),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        match self.token().unwrap().token_type {
            TokenType::Variable(_) => {
                let var: Variable = self.eat().unwrap().try_into()?;
                // try for other Exprs here
                Ok(Expr::Variable(var))
            }
            TokenType::Symbol(_) => Ok(self.parse_call()?),
            TokenType::Exec => {
                self.eat()?;
                self.skip_space()?;
                Ok(self.parse_call()?)
            }
            TokenType::String(_) => {
                todo!("oofers my dude")
            }
            TokenType::ExpandString(_) => {
                todo!("oofers my dude")
            }
            _ => Err(SyntaxError::UnexpectedToken(self.eat().unwrap())),
        }
    }

    fn parse_call(&mut self) -> Result<Expr> {
        let command = self.parse_command()?;
        let mut args = Vec::new();

        loop {
            if let Ok(token) = self.token() {
                match token.token_type {
                    TokenType::Symbol(_)
                    | TokenType::Variable(_)
                    | TokenType::ExpandString(_)
                    | TokenType::Number(_)
                    | TokenType::String(_) => args.push(self.parse_argument()?),
                    TokenType::Space => {
                        self.eat()?;
                    }
                    _ => return Ok(Expr::Call(command, args)),
                }
            } else {
                return Ok(Expr::Call(command, args));
            }
        }
    }

    fn parse_command(&mut self) -> Result<Command> {
        let token = self.eat().unwrap();
        match token.token_type {
            TokenType::ExpandString(text) => Ok(Command::Expand(text.to_string())),
            TokenType::String(text) => Ok(Command::Text(text.to_string())),
            TokenType::Symbol(text) => Ok(Command::Text(text.to_string())),
            _ => Err(SyntaxError::UnexpectedToken(token)),
        }
    }

    fn parse_argument(&mut self) -> Result<Argument> {
        let mut ids = Vec::new();
        let id = self.parse_identifier()?;
        ids.push(id);
        loop {
            if let Ok(token) = self.token() {
                match token.token_type {
                    TokenType::Symbol(_)
                    | TokenType::Variable(_)
                    | TokenType::ExpandString(_)
                    | TokenType::Number(_)
                    | TokenType::String(_) => ids.push(self.parse_identifier()?),
                    _ => return Ok(Argument { parts: ids }),
                };
            } else {
                return Ok(Argument { parts: ids });
            }
        }
    }

    fn parse_identifier(&mut self) -> Result<Identifier> {
        let token = self.eat().unwrap();
        match token.token_type {
            TokenType::String(text) => Ok(Identifier::Text(text.to_string())),
            TokenType::Symbol(text) => Ok(Identifier::Text(text.to_string())),
            TokenType::ExpandString(text) => Ok(Identifier::Expand(text.to_string())),
            TokenType::Variable(name) => Ok(Identifier::Variable(Variable {
                name: name.to_string(),
            })),
            TokenType::Number(number) => Ok(Identifier::Text(number.to_string())),
            _ => Err(SyntaxError::UnexpectedToken(token)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read_to_string;

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
