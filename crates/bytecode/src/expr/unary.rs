use parser::parse::operator::UnaryOperator;

use crate::Opcode;

impl From<UnaryOperator> for Opcode {
    fn from(data: UnaryOperator) -> Self {
        use UnaryOperator::*;
        match data {
            Not => Opcode::Not,
            Negate => Opcode::Neg,
        }
    }
}

#[cfg(test)]
mod test {
    use parser::parse::{
        expr::{atom::AtomicValue, ExprKind},
        operator::UnaryOperator,
    };

    use crate::{
        chunk::Constant,
        test::{assert_bytecode_and_constants, box_node, expr, node},
        Opcode,
    };

    #[test]
    fn generates_unary_op_bytecode() {
        assert_bytecode_and_constants(
            box_node(ExprKind::Unary {
                op: node(UnaryOperator::Negate),
                rhs: expr(AtomicValue::Number(0.0)),
            }),
            vec![Opcode::Constant(0), Opcode::Neg],
            vec![Constant::Number(0.0)],
        );

        assert_bytecode_and_constants(
            box_node(ExprKind::Unary {
                op: node(UnaryOperator::Not),
                rhs: expr(AtomicValue::Number(0.0)),
            }),
            vec![Opcode::Constant(0), Opcode::Not],
            vec![Constant::Number(0.0)],
        );
    }
}
