use crate::bytecode::Chunk;

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub chunk: Chunk,
    pub stack_start: usize,
    pub return_address: usize,
}
