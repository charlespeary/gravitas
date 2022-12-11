use parser::parse::operator::BinaryOperator;

use crate::Opcode;

impl From<BinaryOperator> for Opcode {
    fn from(data: BinaryOperator) -> Self {
        use BinaryOperator::*;
        // +
        match data {
            Addition => Opcode::Add,
            Subtraction => Opcode::Sub,
            Multiplication => Opcode::Mul,
            Division => Opcode::Div,
            Modulo => Opcode::Mod,
            Power => Opcode::Pow,
            Equals => Opcode::Eq,
            NotEquals => Opcode::Ne,
            LesserThan => Opcode::Lt,
            LesserEquals => Opcode::Le,
            GreaterThan => Opcode::Gt,
            GreaterEquals => Opcode::Ge,
            Or => Opcode::Or,
            And => Opcode::And,
        }
    }
}

#[cfg(test)]
mod test {
    use parser::parse::{
        expr::{atom::AtomicValue, ExprKind},
        operator::BinaryOperator,
        Node,
    };

    use crate::{
        chunk::Constant,
        test::{assert_bytecode_and_constants, box_node, expr, node},
        Opcode,
    };

    fn assert_binary_op_bytecode(op: BinaryOperator) {
        let data = box_node(ExprKind::Binary {
            lhs: expr(AtomicValue::Number(0.0)),
            op: node(op),
            rhs: expr(AtomicValue::Number(1.0)),
        });

        assert_bytecode_and_constants(
            data,
            vec![Opcode::Constant(0), Opcode::Constant(1), op.into()],
            vec![Constant::Number(0.0), Constant::Number(1.0)],
        );
    }

    #[test]
    fn generates_binary_operations() {
        use BinaryOperator::*;

        let operators = [
            Addition,
            Subtraction,
            Multiplication,
            Division,
            Modulo,
            Power,
            Equals,
            NotEquals,
            LesserThan,
            LesserEquals,
            GreaterThan,
            GreaterEquals,
            Or,
            And,
        ];

        for op in operators {
            assert_binary_op_bytecode(op);
        }
    }
}
