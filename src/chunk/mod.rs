mod opcode;

pub use crate::chunk::opcode::{Opcode, Value};
use std::slice::Iter;

#[derive(Debug)]
pub struct Chunk {
    code: Vec<Opcode>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: vec![],
            constants: vec![],
        }
    }

    pub fn grow(&mut self, opcode: Opcode) -> usize {
        self.code.push(opcode);
        return self.code.len() - 1;
    }

    pub fn add_constant(&mut self, constant: Value) -> u8 {
        self.constants.push(constant);
        let constant_index = (self.constants.len() - 1) as u8;
        self.grow(Opcode::Constant(constant_index));
        constant_index
    }

    // TODO: Handle longer constants
    pub fn read_constant(&self, index: u8) -> Value {
        *self.constants.get(index as usize).expect("Chunk in wrong state!")
    }
}

impl<'a> IntoIterator for &'a Chunk {
    type Item = <Iter<'a, Opcode> as Iterator>::Item;
    type IntoIter = Iter<'a, Opcode>;

    fn into_iter(self) -> Self::IntoIter {
        self.code.as_slice().into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Chunk code vector should grow in a predictable way
    fn grow() {
        // It returns the index where opcode is stored and
        // grows its size by 1
        let mut chunk = Chunk::new();
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
    /// added to the chunk.code, where u8 is the index of the constant stored in
    /// chunk.constants and constant should be added to the chunk.constants. In return
    /// we get the index of the newly added constant in chunk.constants.
    #[test]
    fn add_constant() {
        let mut chunk = Chunk::new();
        let constant = 10.0;
        let constant_index = chunk.add_constant(constant);
        assert_eq!(chunk.code[0], Opcode::Constant(constant_index));
        assert_eq!(chunk.constants[constant_index as usize], constant);
        assert_eq!(chunk.code.len(), 1);
    }

    /// We read constant at given index in the chunk.constants vector.
    #[test]
    fn read_constant() {
        let mut chunk = Chunk::new();
        let constant = 10.0;
        let constant_index = chunk.add_constant(constant);
        assert_eq!(chunk.read_constant(constant_index), constant);
    }

    /// For now this function should panic when we access the wrong constant.
    ///
    #[test]
    #[should_panic]
    fn read_invalid_constant() {
        let chunk = Chunk::new();
        chunk.read_constant(100);
    }
}