use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, GenerationResult},
    parser::stmt::var::VarStmt,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: String,
    pub depth: usize,
    // Calculated index on the stack
    pub index: usize,
    // Flag to determine whether variable is used inside a closure and needs to be closed
    // in order to be available after it should go off the stack.
    pub closed: bool,
}

impl BytecodeFrom<VarStmt> for BytecodeGenerator {
    fn generate(&mut self, var: &VarStmt) -> GenerationResult {
        let VarStmt { expr, identifier } = var;
        self.generate(expr)?;
        self.state.declare_var(identifier);
        Ok(())
    }
}
