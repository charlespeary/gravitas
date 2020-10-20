use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, GenerationResult},
    parser::expr::{Expr, Grouping},
    parser::stmt::Stmt,
};

mod expr;
mod function;
mod print;
mod var;

impl BytecodeFrom<Stmt> for BytecodeGenerator {
    fn generate(&mut self, stmt: &Stmt) -> GenerationResult {
        match stmt {
            Stmt::Function(fnc) => self.generate(fnc),
            Stmt::Print(print) => self.generate(print),
            Stmt::Var(var) => self.generate(var),
            Stmt::Expr(expr) => self.generate(expr),
        }?;
        Ok(())
    }
}
