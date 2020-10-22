use anyhow::Result;

use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, GenerationResult, Opcode, Patch, PATCH},
    parser::expr::call::Call,
};

impl BytecodeFrom<Call> for BytecodeGenerator {
    fn generate(&mut self, call: &Call) -> GenerationResult {
        Ok(())
    }
}
