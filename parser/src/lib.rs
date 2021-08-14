use std::{collections::VecDeque, convert::TryInto};

mod lexer;
use lexer::{
    token::{Token, TokenType},
    Lexer,
};

mod ast;
use ast::{
    Argument, Ast, Command, Compound, Expression, Identifier, Statement, Variable,
};

mod error;
use error::SyntaxError;

type Result<T> = std::result::Result<T, SyntaxError>;

pub struct Parser {
    tokens: VecDeque<Token>,
}

impl Parser {
    pub fn new(src: String) -> Self {
        Self {
            tokens: Lexer::new(src).collect(),
        }
    }

    #[inline(always)]
    fn token(&self) -> Option<&Token> {
        self.tokens.front()
    }

    #[inline(always)]
    fn eat(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }

    #[inline(always)]
    fn peek(&self, offset: usize) -> Option<&Token> {
        if offset < self.tokens.len() {
            Some(&self.tokens[offset])
        } else {
            None
        }
    }

    pub fn skip_space(&mut self) -> Result<()> {
        while let Some(token) = self.token() {
            if !token.is_space() {
                return Ok(());
            }
            self.eat();
        }
        return Err(SyntaxError::MissingToken);
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
                Some(token) => token,
                None => return Ok(sequence),
            };
            match token.token_type {
                TokenType::Exec
                | TokenType::Variable(_)
                | TokenType::Symbol(_)
                | TokenType::ExpandString(_)
                | TokenType::String(_) => sequence.push(self.parse_compound()?),
                TokenType::Space => drop(self.eat()),
                TokenType::NewLine => drop(self.eat()),
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
                while let Some(token) = self.eat() {
                    match token.token_type {
                        TokenType::Assignment => {
                            self.eat();
                            self.skip_space()?;
                            return Ok(Compound::Statement(Statement::Assignment(var, self.parse_expression()?)));
                        }
                        TokenType::Space => continue,
                        _ => (),
                    }
                }
                Ok(Compound::Expression(Expression::Variable(var)))
            }
            TokenType::Symbol(symbol) => match symbol.as_str() {
                "fn" => todo!("functions not implemented"),
                "loop" => todo!("loop not implemented"),
                "for" => todo!("for not implemented"),
                "while" => todo!("while not implemented"),
                "if" => todo!("if not implemented"),
                _ => Ok(Compound::Expression(self.parse_expression()?)),
            },
            TokenType::Exec | TokenType::String(_) | TokenType::ExpandString(_) => {
                Ok(Compound::Expression(self.parse_expression()?))
            }
            _ => Err(SyntaxError::UnexpectedToken(self.eat().unwrap())),
        }
    }

    /*fn parse_assignment(&mut self) -> Result<Assignment> {
        let token = self.eat().unwrap();
        let variable = match token.token_type {
            TokenType::Variable(variable) => variable,
            _ => return Err(ParseError::UnexpectedToken(token)),
        };

        while self.token().unwrap().is_space() {
            self.eat();
        }
        let expression = self.parse_expression()?;
        Ok(Assignment { variable: Variable{ name: variable }, expression})
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        todo!()
    }*/

    fn parse_expression(&mut self) -> Result<Expression> {
        match self.token().unwrap().token_type {
            TokenType::Variable(_) => {
                let var: Variable = self.eat().unwrap().try_into()?;
                // try for other expressions here
                Ok(Expression::Variable(var))
            },
            TokenType::Symbol(_) => Ok(self.parse_call()?),
            TokenType::Exec => {
                self.eat();
                self.skip_space()?;
                Ok(self.parse_call()?)
            }
            TokenType::String(_) | TokenType::ExpandString(_) => {
                todo!("oofers my dude")
                //Ok(self.parse_call()?)
            }
            _ => Err(SyntaxError::UnexpectedToken(self.eat().unwrap())),
        }
    }

    fn parse_call(&mut self) -> Result<Expression> {
        let command = self.parse_command()?;
        let mut args = Vec::new();

        loop {
            if let Some(token) = self.token() {
                match token.token_type {
                    TokenType::Symbol(_)
                    | TokenType::Variable(_)
                    | TokenType::ExpandString(_)
                    | TokenType::Number(_)
                    | TokenType::String(_) => args.push(self.parse_argument()?),
                    TokenType::Space => {
                        self.eat();
                    }
                    _ => return Ok(Expression::Call(command, args)),
                }
            } else {
                return Ok(Expression::Call(command, args));
            }
        }
    }

    fn parse_command(&mut self) -> Result<Command> {
        let token = self.eat().unwrap();
        match token.token_type {
            TokenType::ExpandString(text) => Ok(Command::Expand(text)),
            TokenType::String(text) => Ok(Command::Text(text)),
            TokenType::Symbol(text) => Ok(Command::Text(text)),
            _ => Err(SyntaxError::UnexpectedToken(token)),
        }
    }

    fn parse_argument(&mut self) -> Result<Argument> {
        let mut ids = Vec::new();
        let id = self.parse_identifier()?;
        ids.push(id);
        loop {
            if let Some(token) = self.token() {
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
            TokenType::String(text) => Ok(Identifier::Text(text)),
            TokenType::Symbol(text) => Ok(Identifier::Text(text)),
            TokenType::ExpandString(text) => Ok(Identifier::Expand(text)),
            TokenType::Variable(name) => Ok(Identifier::Variable(Variable { name })),
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
