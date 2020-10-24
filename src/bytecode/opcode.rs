use derive_more::Display;

#[derive(Debug, PartialOrd, PartialEq, Eq, Copy, Clone, Display, Hash)]
pub enum Opcode {
    // Inline values
    True,
    False,
    Null,
    // Constant
    Constant(usize),
    // Negation
    Not,
    Negate,
    // Binary
    Add,
    Subtract,
    Multiply,
    Divide,
    // Comparison
    Compare,
    BangEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    // Jumps
    JumpIfFalse(usize),
    JumpForward(usize),
    JumpBack(usize),
    // Expressions
    Return,
    Break(usize),
    // Block holds number of variables declared inside to drop
    Block(usize),
    // Side effects
    Print,
    PopN(usize),
    // Get resource
    Get,
    // Assign value to a given resource
    Assign,
    // Calls
    Call,
}

impl Opcode {
    pub fn patch(self, index: usize) -> Self {
        match self {
            Opcode::JumpIfFalse(_) => Opcode::JumpIfFalse(index),
            Opcode::JumpForward(_) => Opcode::JumpForward(index),
            Opcode::JumpBack(_) => Opcode::JumpBack(index),
            Opcode::Block(_) => Opcode::Block(index),
            Opcode::Break(_) => Opcode::Break(index),
            _ => unreachable!("Tried to patch invalid opcode"),
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

#[cfg(test)]
mod test {
    use pretty_assertions::{assert_eq, assert_ne};

    use super::*;
}
