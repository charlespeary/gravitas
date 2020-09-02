#[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
pub enum Opcode {
    Constant(u8),
    Return,
}