use crate::parser::Token;

#[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
pub enum Opcode {
    // Values
    True,
    False,
    Null,
    Constant(u8),
    // Negation stuff
    Negate,
    // binary operators
    Add,
    Subtract,
    Multiply,
    Divide,
    // Comparison
    BangEqual,
    Equal,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    //
    Return,
}

impl From<Token> for Opcode {
    fn from(token: Token) -> Self {
        match token {
            Token::Plus => Opcode::Add,
            Token::Minus => Opcode::Subtract,
            Token::Star => Opcode::Multiply,
            Token::Divide => Opcode::Divide,
            Token::BangEqual => Opcode::BangEqual,
            Token::Equal => Opcode::Equal,
            Token::Less => Opcode::Less,
            Token::LessEqual => Opcode::LessEqual,
            Token::Greater => Opcode::Greater,
            Token::GreaterEqual => Opcode::GreaterEqual,
            _ => panic!("Can't transform {} into opcode.", token),
        }
    }
}

impl From<bool> for Opcode {
    fn from(bool: bool) -> Self {
        match bool {
            true => Opcode::True,
            false => Opcode::False,
        }
    }
}
