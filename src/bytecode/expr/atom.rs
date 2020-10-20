use crate::bytecode::GenerationResult;
use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, Opcode, Value},
    parser::expr::Atom,
};

impl BytecodeFrom<Atom> for BytecodeGenerator {
    fn generate(&mut self, atom: &Atom) -> GenerationResult {
        match atom {
            Atom::Number(num) => {
                self.chunk.add_constant(Value::Number(*num));
            }
            Atom::Bool(bool) => {
                self.emit_code((*bool).into());
            }
            Atom::Null => {
                self.emit_code(Opcode::Null);
            }
            Atom::Text(string) => {
                self.chunk.add_constant(Value::String(string.clone()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::bytecode::test::generate_bytecode;

    use super::*;

    #[quickcheck]
    fn expr_atom_numbers(a: f64) {
        let ast = Atom::Number(a);
        let (chunk, bytecode) = generate_bytecode(ast);

        assert_eq!(bytecode, vec![Opcode::Constant(0)]);
        assert_eq!(*chunk.read_constant(0), Value::Number(a));
    }

    #[test]
    fn expr_atom_boolean() {
        let ast = Atom::Bool(true);
        let (_, bytecode) = generate_bytecode(ast);
        assert_eq!(bytecode, vec![Opcode::True]);

        let ast = Atom::Bool(false);
        let (_, bytecode) = generate_bytecode(ast);
        assert_eq!(bytecode, vec![Opcode::False]);
    }

    #[test]
    fn expr_atom_null() {
        let ast = Atom::Null;
        let (_, bytecode) = generate_bytecode(ast);
        assert_eq!(bytecode, vec![Opcode::Null]);
    }

    #[quickcheck]
    fn expr_atom_text(text: String) {
        let ast = Atom::Text(text.clone());
        let (chunk, bytecode) = generate_bytecode(ast);
        assert_eq!(bytecode, vec![Opcode::Constant(0)]);
        assert_eq!(*chunk.read_constant(0), Value::String(text));
    }
}
