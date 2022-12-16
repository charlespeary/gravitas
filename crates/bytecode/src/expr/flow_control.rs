#[cfg(test)]
mod test {
    use parser::parse::expr::{atom::AtomicValue, ExprKind};

    use crate::{
        chunk::Constant,
        test::{assert_bytecode_and_constants, box_node, expr, expr_stmt},
        Opcode,
    };

    #[test]
    fn generates_while_loop_bytecode() {
        let while_loop = expr_stmt(box_node(ExprKind::While {
            condition: expr(AtomicValue::Boolean(true)),
            body: box_node(ExprKind::Block {
                stmts: vec![expr_stmt(expr(AtomicValue::Number(0.0)))],
                return_expr: None,
            }),
        }));

        assert_bytecode_and_constants(
            while_loop,
            vec![
                Opcode::Constant(0),
                Opcode::Jif(4),
                Opcode::Constant(1),
                Opcode::Block(0),
                Opcode::Jp(-3),
                Opcode::Null,
            ],
            vec![Constant::Bool(true), Constant::Number(0.0)],
        );
    }
}
