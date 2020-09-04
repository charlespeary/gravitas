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

pub type Value = f64;