use crate::bytecode::GenerationResult;
use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, Opcode, Value},
    parser::expr::{Atom, Binary},
};

impl BytecodeFrom<Binary> for BytecodeGenerator {
    fn generate(&mut self, binary: &Binary) -> GenerationResult {
        let Binary {
            left,
            right,
            operator,
        } = binary;
        self.generate(left)?;
        self.generate(right)?;
        self.emit_code(operator.clone().into());
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::{
        bytecode::test::generate_bytecode,
        parser::expr::{binary::Operator, Expr},
    };

    use super::*;

    #[quickcheck]
    fn expr_binary(a: f64, b: f64) {
        macro_rules! test_binary_with_operators (
            ($a: expr, $b: expr, $($operator: expr),*) => {
                $(
                    {
                        let ast = Binary {
                            left: Box::new(Expr::Atom(Atom::Number($a))),
                            operator: $operator,
                            right: Box::new(Expr::Atom(Atom::Number($b))),
                        };
                        let (chunk, bytecode) = generate_bytecode(ast);
                        assert_eq!(
                                bytecode,
                                vec![Opcode::Constant(0), Opcode::Constant(1), $operator.into()]
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
