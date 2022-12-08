use common::Symbol;

use crate::chunk::Chunk;

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: Symbol,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Class {
    pub name: Symbol,
    pub constructor: Function,
    pub methods: Vec<Function>,
    pub super_class: Option<Box<Class>>,
}
