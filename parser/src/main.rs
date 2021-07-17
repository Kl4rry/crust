use std::fs::read_to_string;

pub mod token;
pub mod lexer;
pub mod parser;

fn main() {
    let src = read_to_string("test.crust").unwrap();
    let lexer = lexer::Lexer::new(src);
    for token in lexer {
        println!("{:?}", token.token_type);
    }
}
