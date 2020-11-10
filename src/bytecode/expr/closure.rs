use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, Callable, Chunk, GenerationResult, Opcode, Patch, PATCH, Value},
    parser::{Expr, expr::Closure as ClosureExpr},
};

#[derive(Clone, PartialEq, PartialOrd)]
pub struct Closure {
    pub  chunk: Chunk,
    pub  arity: usize,
}

impl BytecodeFrom<ClosureExpr> for BytecodeGenerator {
    fn generate(&mut self, closure: &ClosureExpr) -> GenerationResult {
        let ClosureExpr { body, params } = closure;

        let mut emitter = self.child();
        emitter.state.in_closure = true;

        for param in params.clone() {
            emitter.declare(param.val);
        }

        match *body.clone() {
            Expr::Block(block) => {
                for item in &block.body {
                    emitter.generate(item)?;
                }
                // Add explicit return with null if user didn't
                if !emitter.state.function_returned {
                    emitter.emit_code(Opcode::Null);
                    emitter.emit_code(Opcode::Return);
                }
            }
            b => {
                emitter.generate(&b)?;
                emitter.emit_code(Opcode::Return);
            }
        }

        let lambda = Closure {
            arity: params.len(),
            chunk: emitter.chunk,
        };

        self.add_constant(Value::Callable(Callable::Closure(lambda)));

        Ok(())
    }
}