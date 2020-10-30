use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, GenerationResult, Opcode},
    parser::stmt::expr::ExprStmt,
};

impl BytecodeFrom<ExprStmt> for BytecodeGenerator {
    fn generate(&mut self, expr: &ExprStmt) -> GenerationResult {
        self.generate(&expr.expr)?;
        self.emit_code(Opcode::PopN(1));
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::{
        bytecode::{test::generate_bytecode, Opcode, Value},
        parser::{
            expr::{atom::Atom, Expr},
            stmt::Stmt,
        },
    };

    use super::*;

    #[test]
    fn stmt_expr() {
        // Stmt::Expr is just a side effect to kick off the expression stored inside it.
        let ast = Stmt::Expr(ExprStmt {
            expr: Expr::Atom(Atom::Number(10.0)),
        });

        let (chunk, bytecode) = generate_bytecode(ast);

        assert_eq!(bytecode, vec![Opcode::Constant(0), Opcode::PopN(1)]);
        assert_eq!(chunk.read_constant(0), &Value::Number(10.0));
    }
}
