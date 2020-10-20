use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, GenerationResult, Opcode},
    parser::stmt::function::FunctionStmt,
};

impl BytecodeFrom<FunctionStmt> for BytecodeGenerator {
    fn generate(&mut self, fnc: &FunctionStmt) -> GenerationResult {
        // self.generate(expr)?;
        Ok(())
    }
}
