use crate::Opcode;
use common::{Number, Symbol};

#[derive(Clone, Debug, PartialEq)]
pub enum Constant {
    Number(Number),
    String(Symbol),
    Bool(bool),
}

pub type ConstantIndex = usize;

#[derive(Debug, Clone, Default)]
pub struct Chunk {
    pub codes: Vec<Opcode>,
    pub constants: Vec<Constant>,
}

impl Chunk {
    pub fn read(&self, index: ConstantIndex) -> Constant {
        self.constants
            .get(index)
            .expect("Constant out of bounds.")
            .clone()
    }

    pub fn write(&mut self, constant: Constant) -> ConstantIndex {
        let length = self.constants.len();

        if length > std::u16::MAX as usize {
            panic!("This program got too big!");
        };

        self.constants.push(constant);
        length
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_from_chunk() {
        let chunk = Chunk {
            codes: vec![],
            constants: vec![
                Constant::Number(10.0),
                Constant::Bool(false),
                Constant::Bool(true),
            ],
        };

        assert_eq!(chunk.read(0), Constant::Number(10.0));
        assert_eq!(chunk.read(1), Constant::Bool(false));
        assert_eq!(chunk.read(2), Constant::Bool(true));
    }

    #[test]
    fn write_to_chunk() {
        let mut chunk = Chunk::default();

        assert_eq!(chunk.write(Constant::Bool(true)), 0);
        assert_eq!(chunk.write(Constant::Number(32.0)), 1);
        assert_eq!(chunk.write(Constant::Bool(false)), 2)
    }
}
