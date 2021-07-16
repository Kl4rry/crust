use std::fs::read_to_string;

pub mod token;
use token::Token;
pub mod lexer;

fn main() {
    let src = read_to_string("test.crust").unwrap();
    let mut lexer = lexer::Lexer::new(src);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        tokens.push(token);
        if *tokens.last().unwrap() == Token::EOF {
            break;
        }
    }
    println!("{:?}", tokens);
}
