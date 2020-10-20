use std::slice::Iter;

use derive_more::Display;

use crate::bytecode::{Opcode, Value};

#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
pub struct Chunk {
    pub code: Vec<Opcode>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn grow(&mut self, opcode: Opcode) -> usize {
        let inserted_at = self.code.len();
        self.code.push(opcode);
        inserted_at
    }

    pub fn size(&self) -> usize {
        self.code.len()
    }

    pub fn add_constant(&mut self, constant: Value) -> usize {
        self.constants.push(constant);
        let constant_index = self.constants.len() - 1;
        self.grow(Opcode::Constant(constant_index));
        constant_index
    }

    pub fn read_constant(&self, index: usize) -> &Value {
        self.constants.get(index).expect("Chunk in wrong state!")
    }
}

impl<'a> IntoIterator for &'a Chunk {
    type Item = <Iter<'a, Opcode> as Iterator>::Item;
    type IntoIter = Iter<'a, Opcode>;

    fn into_iter(self) -> Self::IntoIter {
        self.code.as_slice().iter()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    /// Chunk code vector should grow in a predictable way
    fn grow() {
        // It returns the index where opcode is stored and
        // grows its size by 1
        let mut chunk = Chunk::default();
        assert_eq!(chunk.grow(Opcode::Return), 0);
        assert_eq!(chunk.code.len(), 1);

        // The opcode we insert is at the exact position that's
        // returned by the grow() function
        let opcode = Opcode::Constant(100);
        let index = chunk.grow(opcode.clone());
        assert_eq!(index, 1);
        assert_eq!(opcode, chunk.code[index]);
        assert_eq!(chunk.code.len(), 2);
    }

    /// When we add a constant, an Opcode::Constant(u8) should be
    /// added to the bytecode.code, where u8 is the index of the constant stored in
    /// bytecode.constants and constant should be added to the bytecode.constants. In return
    /// we get the index of the newly added constant in bytecode.constants.
    #[test]
    fn add_constant() {
        let mut chunk = Chunk::default();
        let constant = Value::Number(10.0);
        let constant_index = chunk.add_constant(constant.clone());
        assert_eq!(chunk.code[0], Opcode::Constant(constant_index));
        assert_eq!(chunk.constants[constant_index as usize], constant);
        assert_eq!(chunk.code.len(), 1);
    }

    /// We read constant at given index in the bytecode.constants vector.
    #[test]
    fn read_constant() {
        let mut chunk = Chunk::default();
        let constant = Value::Number(10.0);
        let constant_index = chunk.add_constant(constant.clone());
        assert_eq!(chunk.read_constant(constant_index), &constant);
    }

    /// For now this function should panic when we access the wrong constant.
    ///
    #[test]
    #[should_panic]
    fn read_invalid_constant() {
        let chunk = Chunk::default();
        chunk.read_constant(100);
    }
}
