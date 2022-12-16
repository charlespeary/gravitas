use crate::{BytecodeFrom, BytecodeGenerationResult, BytecodeGenerator, Opcode};
use parser::parse::stmt::{Stmt, StmtKind};

mod var;

impl BytecodeFrom<Stmt> for BytecodeGenerator {
    fn generate(&mut self, stmt: Stmt) -> BytecodeGenerationResult {
        match *stmt.kind {
            StmtKind::Expression { expr } => {
                self.generate(expr)?;
            }
            StmtKind::VariableDeclaration { name, expr } => {
                self.generate(expr)?;
                self.state.declare_var(name);
            }
            StmtKind::FunctionDeclaration { name, params, body } => {}
            StmtKind::ClassDeclaration {
                name,
                super_class,
                methods,
            } => {}
        }
        Ok(())
    }
}
