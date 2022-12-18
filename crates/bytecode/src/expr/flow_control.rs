#[cfg(test)]
mod test {
    use parser::parse::expr::{atom::AtomicValue, ExprKind};

    use crate::{
        chunk::Constant,
        test::{assert_bytecode_and_constants, box_node, expr, expr_stmt, node},
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
                Opcode::Null,
                Opcode::Block(0),
                Opcode::Jp(-4),
                Opcode::Null,
            ],
            vec![Constant::Bool(true), Constant::Number(0.0)],
        );
    }

    #[test]
    fn generates_if_bytecode() {
        // With else
        assert_bytecode_and_constants(
            box_node(ExprKind::If {
                condition: expr(AtomicValue::Boolean(true)),
                body: expr(AtomicValue::Boolean(true)),
                else_expr: Some(expr(AtomicValue::Boolean(false))),
            }),
            vec![
                Opcode::Constant(0),
                Opcode::Jif(3),
                Opcode::Constant(1),
                Opcode::Jp(1),
                Opcode::Constant(2),
            ],
            vec![
                Constant::Bool(true),
                Constant::Bool(true),
                Constant::Bool(false),
            ],
        );

        // Without else
        assert_bytecode_and_constants(
            box_node(ExprKind::If {
                condition: expr(AtomicValue::Boolean(true)),
                body: expr(AtomicValue::Boolean(false)),
                else_expr: None,
            }),
            vec![
                Opcode::Constant(0),
                Opcode::Jif(2),
                Opcode::Constant(1),
                Opcode::Jp(0),
            ],
            vec![Constant::Bool(true), Constant::Bool(false)],
        );
    }

    #[test]
    fn generates_break_bytecode() {
        let data = box_node(ExprKind::While {
            condition: expr(AtomicValue::Boolean(true)),
            body: box_node(ExprKind::Block {
                stmts: vec![expr_stmt(box_node(ExprKind::Break {
                    return_expr: Some(expr(AtomicValue::Number(5.0))),
                }))],
                return_expr: None,
            }),
        });

        assert_bytecode_and_constants(
            data,
            vec![
                Opcode::Constant(0),
                Opcode::Jif(5),
                Opcode::Constant(1),
                Opcode::Break(4),
                Opcode::Null,
                Opcode::Block(0),
                Opcode::Jp(-5),
                Opcode::Null,
            ],
            vec![Constant::Bool(true), Constant::Number(5.0)],
        );
    }

    #[test]
    fn generates_continue_bytecode() {
        let data = box_node(ExprKind::While {
            condition: expr(AtomicValue::Boolean(true)),
            body: box_node(ExprKind::Block {
                stmts: vec![expr_stmt(box_node(ExprKind::Continue))],
                return_expr: None,
            }),
        });

        assert_bytecode_and_constants(
            data,
            vec![
                Opcode::Constant(0),
                Opcode::Jif(4),
                Opcode::Jp(-1),
                Opcode::Null,
                Opcode::Block(0),
                Opcode::Jp(-4),
                Opcode::Null,
            ],
            vec![Constant::Bool(true)],
        );
    }
}
