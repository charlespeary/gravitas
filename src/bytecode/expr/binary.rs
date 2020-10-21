use crate::bytecode::GenerationResult;
use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, Opcode, Value},
    parser::{
        expr::{Atom, Binary},
        operator::Operator,
    },
};

fn bin_op_to_opcode(operator: Operator) -> Opcode {
    match operator {
        Operator::Plus => Opcode::Add,
        Operator::Minus => Opcode::Subtract,
        Operator::Multiply => Opcode::Multiply,
        Operator::Divide => Opcode::Divide,
        Operator::Assign => Opcode::Assign,
        Operator::Compare => Opcode::Compare,
        Operator::Less => Opcode::Less,
        Operator::LessEqual => Opcode::LessEqual,
        Operator::Greater => Opcode::Greater,
        Operator::GreaterEqual => Opcode::GreaterEqual,
        // Parser will ensure that no incorrect operators will get there
        _ => unreachable!(),
    }
}

impl BytecodeFrom<Binary> for BytecodeGenerator {
    fn generate(&mut self, binary: &Binary) -> GenerationResult {
        let Binary { lhs, rhs, operator } = binary;
        self.generate(lhs)?;
        self.generate(rhs)?;

        self.emit_code(bin_op_to_opcode(*operator));
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::{
        bytecode::test::generate_bytecode,
        parser::{expr::Expr, operator::Operator},
    };

    use super::*;

    #[quickcheck]
    fn expr_binary(a: f64, b: f64) {
        macro_rules! test_binary_with_operators (
            ($a: expr, $b: expr, $($operator: expr),*) => {
                $(
                    {
                        let ast = Binary {
                            lhs: Box::new(Expr::Atom(Atom::Number($a))),
                            operator: $operator,
                            rhs: Box::new(Expr::Atom(Atom::Number($b))),
                        };
                        let (chunk, bytecode) = generate_bytecode(ast);
                        assert_eq!(
                                bytecode,
                                vec![Opcode::Constant(0), Opcode::Constant(1), bin_op_to_opcode($operator)]
                         );
                         assert_eq!(
                                (chunk.read_constant(0), chunk.read_constant(1)),
                                (&Value::Number(a), &Value::Number(b))
                       );
                    }
                )
            *}
        );

        test_binary_with_operators!(
            a,
            b,
            Operator::Plus,
            Operator::Minus,
            Operator::Multiply,
            Operator::Divide
        );
    }
}
