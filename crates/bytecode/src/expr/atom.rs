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
                is_assignment,
                properties,
                name,
            } => {
                // TODO: make it for work object's properties
                let var_address = self.state.find_var(&name);
                self.write_constant(var_address.into());

                // We evaluate the address if it's not used in an assignment context
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
    use parser::parse::{
        expr::{atom::AtomicValue, ExprKind},
        stmt::{Stmt, StmtKind},
        Node,
    };

    use crate::{chunk::Constant, test::assert_bytecode_and_constants, MemoryAddress, Opcode};

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
                Node {
                    kind: Box::new(StmtKind::VariableDeclaration {
                        name: "foo".to_owned(),
                        expr: Node {
                            kind: Box::new(ExprKind::Atom(AtomicValue::Text("bar".to_owned()))),
                            span: 0..0,
                        },
                    }),
                    span: 0..0,
                },
                Node {
                    kind: Box::new(StmtKind::Expression {
                        expr: Node {
                            kind: Box::new(ExprKind::Atom(AtomicValue::Identifier {
                                name: "foo".to_owned(),
                                is_assignment: false,
                                properties: vec![],
                            })),
                            span: 0..0,
                        },
                    }),
                    span: 0..0,
                },
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
