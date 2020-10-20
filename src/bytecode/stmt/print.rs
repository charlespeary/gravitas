use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, GenerationResult, Opcode},
    parser::stmt::print::PrintStmt,
};

impl BytecodeFrom<PrintStmt> for BytecodeGenerator {
    fn generate(&mut self, var: &PrintStmt) -> GenerationResult {
        let PrintStmt { expr } = var;
        self.generate(expr)?;
        self.emit_code(Opcode::Print);
        Ok(())
    }
}
