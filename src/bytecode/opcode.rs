use ordered_float::NotNan;

use crate::parser::Token;

#[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
pub enum Opcode {
    Constant(u8),
    Negate,
    // binary operators
    Add,
    Subtract,
    Multiply,
    Divide,
    //
    Return,
}

pub type Value = NotNan<f64>;

impl From<Token> for Opcode {
    fn from(token: Token) -> Self {
        match token {
            Token::Plus => Opcode::Add,
            Token::Minus => Opcode::Subtract,
            Token::Star => Opcode::Multiply,
            Token::Divide => Opcode::Divide,
            _ => panic!("Can't transform {} into opcode.", token),
        }
    }
}
