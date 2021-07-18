use common::Symbol;

use crate::chunk::Chunk;

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: Symbol,
}
