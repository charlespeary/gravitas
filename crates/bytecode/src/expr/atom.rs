use parser::parse::expr::atom::AtomicValue;

use crate::{chunk::Constant, BytecodeFrom, BytecodeGenerationResult, BytecodeGenerator};

impl BytecodeFrom<AtomicValue> for BytecodeGenerator {
    fn generate(&mut self, data: AtomicValue) -> BytecodeGenerationResult {
        match data {
            AtomicValue::Boolean(bool) => {
                self.write_constant(Constant::Bool(bool));
            }
            AtomicValue::Number(number) => {
                self.write_constant(Constant::Number(number));
            }
            AtomicValue::Text(text) => {
                self.write_constant(Constant::String(text));
            }
            AtomicValue::Identifier(ProgramText) => {
                // TODO: lookup variable's address
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use parser::parse::expr::atom::AtomicValue;

    use crate::{chunk::Constant, test::assert_bytecode_and_constants, Opcode};

    #[test]
    fn generates_atoms() {
        assert_bytecode_and_constants(
            AtomicValue::Boolean(true),
            vec![Opcode::Constant(0)],
            vec![Constant::Bool(true)],
        );

        assert_bytecode_and_constants(
            AtomicValue::Boolean(false),
            vec![Opcode::Constant(0)],
            vec![Constant::Bool(false)],
        );

        assert_bytecode_and_constants(
            AtomicValue::Number(0.0),
            vec![Opcode::Constant(0)],
            vec![Constant::Number(0.0)],
        );

        assert_bytecode_and_constants(
            AtomicValue::Text("foo".to_owned()),
            vec![Opcode::Constant(0)],
            vec![Constant::String("foo".to_owned())],
        );
    }
}
