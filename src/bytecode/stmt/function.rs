use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, Callable, Chunk, GenerationResult, Opcode, Value},
    parser::stmt::function::FunctionStmt,
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Function {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: String,
}

impl Into<Value> for Function {
    fn into(self) -> Value {
        Value::Callable(Callable::Function(self))
    }
}

impl BytecodeFrom<FunctionStmt> for BytecodeGenerator {
    fn generate(&mut self, fnc: &FunctionStmt) -> GenerationResult {
        let FunctionStmt { name, params, body } = fnc;
        let mut emitter = self.child();

        // Declare parameters, so they are visible in the body scope
        for param in params.clone() {
            emitter.declare(param.val);
        }

        // Declare function, so we can allow recursive calls.
        // It happens after the parameters, because arguments are evaluated first, then comes the caller value onto the stack.
        emitter.declare(name.clone());

        // We don't want to evaluate block expression, only its items
        for item in &body.body {
            emitter.generate(item)?;
        }

        // Add explicit return with null if user didn't
        if !emitter.state.function_returned {
            emitter.emit_code(Opcode::Null);
            emitter.emit_code(Opcode::Return);
        }

        let function_chunk = emitter.chunk;
        let function = Value::Callable(Callable::Function(Function {
            arity: params.len(),
            chunk: function_chunk,
            name: name.clone(),
        }));

        self.add_constant(function);
        self.declare(name.clone());

        Ok(())
    }
}
