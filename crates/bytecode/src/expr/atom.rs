use parser::parse::expr::atom::AtomicValue;

use crate::{chunk::Constant, BytecodeFrom, BytecodeGenerationResult, BytecodeGenerator, Opcode};

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
            AtomicValue::Identifier {
                name,
                is_assignment,
            } => {
                let var_address = self
                    .state
                    .find_var_address(&name)
                    .expect("Analyzer takes care of undefined variables");

                self.write_constant(var_address.into());

                if !is_assignment {
                    self.write_opcode(Opcode::Get);
                }
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use parser::parse::expr::atom::AtomicValue;

    use crate::{
        chunk::Constant,
        test::{assert_bytecode_and_constants, declare_var, expr, expr_stmt},
        MemoryAddress, Opcode,
    };

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

    #[test]
    fn generates_variable_identifiers() {
        // We need to declare variable first
        // otherwise generator won't find it inside the scope and will panic.
        // Static analysis ensures that we won't get AST that allows it.
        assert_bytecode_and_constants(
            vec![
                declare_var("foo".to_owned(), expr(AtomicValue::Text("bar".to_owned()))),
                expr_stmt(expr(AtomicValue::Identifier {
                    name: "foo".to_owned(),
                    is_assignment: false,
                })),
            ],
            vec![Opcode::Constant(0), Opcode::Constant(1), Opcode::Get],
            vec![
                Constant::String("bar".to_owned()),
                Constant::MemoryAddress(MemoryAddress::Local(0)),
            ],
        );
    }

    #[test]
    fn generates_object_properties() {}
}
