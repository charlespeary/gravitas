mod token;


use anyhow::Result;
use logos::Logos;
pub use token::Token;

pub fn compile(code: &str) {
    let tokens = Token::lexer(code);
    for token in tokens {
        println!("{:#?}", token);
    }
}