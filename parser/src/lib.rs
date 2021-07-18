use std::{collections::VecDeque, fs::read_to_string};

mod lexer;
use lexer::{
    token::{Token, TokenType},
    Lexer,
};

mod ast;
use ast::{Argument, Ast, Call, Compound, Expression, Identifier, Variable};

mod error;
use error::ParseError;

type Result<T> = std::result::Result<T, ParseError>;

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
                TokenType::Argument(_) => sequence.push(self.parse_compound()?),
                TokenType::Space => drop(self.eat()),
                TokenType::NewLine => drop(self.eat()),
                _ => return Err(ParseError::UnexpectedToken(self.eat().unwrap())),
            };
        }
    }

    fn parse_compound(&mut self) -> Result<Compound> {
        match self.token().unwrap().token_type {
            //parse ifs and loops and fn and other statements here
            TokenType::Argument(_) => Ok(Compound::Expression(self.parse_expression()?)),
            _ => unreachable!(),
        }
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        match self.token().unwrap().token_type {
            TokenType::Argument(_) => Ok(Expression::Call(self.parse_call()?)),
            _ => unreachable!(),
        }
    }

    fn parse_call(&mut self) -> Result<Call> {
        let mut args = Vec::new();
        args.push(self.parse_argument()?);

        loop {
            if let Some(token) = self.token() {
                match token.token_type {
                    // TODO way more tokens are allowed in these plz add them
                    TokenType::Argument(_) => args.push(self.parse_argument()?),
                    TokenType::Glob(_) => args.push(self.parse_argument()?),
                    TokenType::Variable(_) => args.push(self.parse_argument()?),
                    TokenType::Space => {
                        self.eat();
                    },
                    _ => return Ok(Call { args }),
                }
            } else {
                return Ok(Call { args });
            }
        }
    }

    fn parse_argument(&mut self) -> Result<Argument> {
        let mut ids = Vec::new();
        let id = self.parse_identifier()?;
        ids.push(id);
        loop {
            if let Some(token) = self.token() {
                match token.token_type {
                    TokenType::Argument(_) => ids.push(self.parse_identifier()?),
                    TokenType::Glob(_) => ids.push(self.parse_identifier()?),
                    TokenType::Variable(_) => ids.push(self.parse_identifier()?),
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
            TokenType::Argument(text) => Ok(Identifier::Text(text)),
            TokenType::Glob(text) => Ok(Identifier::Glob(text)),
            TokenType::Variable(name) => Ok(Identifier::Variable(Variable { name })),
            _ => Err(ParseError::UnexpectedToken(token)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_test() {
        println!("{}", std::mem::size_of::<Ast>());
        let src = read_to_string("test.crust").unwrap();
        let mut parser = Parser::new(src);
        let ast = parser.parse();
        match ast {
            Ok(ast) => println!("{:#?}", ast),
            Err(error) => eprintln!("{}", error),
        }
    }
}
