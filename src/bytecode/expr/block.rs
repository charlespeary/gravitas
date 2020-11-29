use crate::{
    bytecode::{state::ScopeType, BytecodeFrom, BytecodeGenerator, GenerationResult, Opcode},
    parser::expr::Block,
};

impl BytecodeFrom<Block> for BytecodeGenerator {
    fn generate(&mut self, block: &Block) -> GenerationResult {
        let Block { body, final_expr } = block;
        self.state.enter_scope(ScopeType::Block);

        for item in body {
            self.generate(item)?;
        }

        match final_expr {
            Some(expr) => {
                self.generate(expr)?;
            }
            _ => {
                self.emit_code(Opcode::Null);
            }
        }

        let declared = self.state.declared();

        self.emit_code(Opcode::Block(declared));

        self.state.leave_scope();

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::{
        bytecode::{test::generate_bytecode, Value},
        parser::{
            expr::{atom::Atom, Expr},
            stmt::{var::VarStmt, Stmt},
        },
    };

    use super::*;

    #[test]
    fn expr_block_no_final_expr() {
        let ast = Block {
            body: vec![Stmt::Var(VarStmt {
                identifier: String::from("foo"),
                expr: Expr::Atom(Atom::Number(10.0)),
            })],
            final_expr: None,
        };

        let (chunk, bytecode) = generate_bytecode(ast);

        assert_eq!(
            bytecode,
            vec![Opcode::Constant(0), Opcode::Null, Opcode::Block(1)]
        );
        assert_eq!(chunk.read_constant(0), &Value::Number(10.0));
    }

    #[test]
    fn expr_block_final_expr() {
        let ast = Block {
            body: vec![],
            final_expr: Some(Box::new(Expr::Atom(Atom::Number(10.0)))),
        };

        let (chunk, bytecode) = generate_bytecode(ast);

        // When no variables are created inside the block, then no Opcode::Block is added
        // because there are no variables to drop
        assert_eq!(bytecode, vec![Opcode::Constant(0)]);
        assert_eq!(chunk.read_constant(0), &Value::Number(10.0));

        let ast = Block {
            body: vec![Stmt::Var(VarStmt {
                identifier: String::from("foo"),
                expr: Expr::Atom(Atom::Null),
            })],
            final_expr: Some(Box::new(Expr::Atom(Atom::Number(10.0)))),
        };

        let (chunk, bytecode) = generate_bytecode(ast);

        // Opcode::Block is added whenever we declare variables inside the block, so they are dropped
        // at the end of the block.
        assert_eq!(
            bytecode,
            vec![Opcode::Null, Opcode::Constant(0), Opcode::Block(1)]
        );
        assert_eq!(chunk.read_constant(0), &Value::Number(10.0));
    }
}
