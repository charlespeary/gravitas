use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, Chunk, GenerationResult, Opcode},
    parser::stmt::function::FunctionStmt,
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Function {
    arity: usize,
    chunk: Chunk,
    name: String,
}

impl BytecodeFrom<FunctionStmt> for BytecodeGenerator {
    fn generate(&mut self, fnc: &FunctionStmt) -> GenerationResult {
        let FunctionStmt { name, params, body } = fnc;
        let mut emitter = BytecodeGenerator::new();
        for param in params.clone() {
            emitter.add_local(param.val);
        }
        emitter.generate(body)?;
        let function_chunk = emitter.chunk;
        let function = Function {
            arity: params.len(),
            chunk: function_chunk,
            name: name.clone(),
        };
        Ok(())
    }
}
