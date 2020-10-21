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
        let FunctionStmt { name, args, body } = fnc;
        let mut emitter = BytecodeGenerator::new();
        for arg in args.clone() {
            emitter.add_local(arg.val);
        }
        emitter.generate(body)?;
        let function_chunk = emitter.chunk;
        let function = Function {
            arity: args.len(),
            chunk: function_chunk,
            name: name.clone(),
        };
        Ok(())
    }
}
