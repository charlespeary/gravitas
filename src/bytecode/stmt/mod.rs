use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, GenerationResult},
    parser::stmt::Stmt,
};

pub mod expr;
pub mod function;
pub mod var;

impl BytecodeFrom<Stmt> for BytecodeGenerator {
    fn generate(&mut self, stmt: &Stmt) -> GenerationResult {
        match stmt {
            Stmt::Function(fnc) => self.generate(fnc),
            Stmt::Var(var) => self.generate(var),
            Stmt::Expr(expr) => self.generate(expr),
        }?;
        Ok(())
    }
}
