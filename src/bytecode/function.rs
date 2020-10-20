use crate::bytecode::Chunk;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Function {
    arity: usize,
    chunk: Chunk,
    name: String,
}
