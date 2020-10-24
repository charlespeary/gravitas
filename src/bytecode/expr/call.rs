use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, GenerationResult, Opcode, Patch, PATCH},
    parser::expr::call::{Call, Return},
};

impl BytecodeFrom<Call> for BytecodeGenerator {
    fn generate(&mut self, call: &Call) -> GenerationResult {
        let Call { caller, args } = call;

        // Evaluate each argument, so they are present on the stack and ready to be consumed by function
        for arg in &args.0 {
            self.generate(arg)?;
        }
        // Evaluate the expression, so we get the value with function or method we want to call
        // Caller needs to be evaluated after the arguments, so we can pop the function from the stack
        // and check the arity in order to pop correct number of values from stack.
        self.generate(caller)?;
        // Emit Opcode to kick off the call
        self.emit_code(Opcode::Call);
        Ok(())
    }
}

impl BytecodeFrom<Return> for BytecodeGenerator {
    fn generate(&mut self, ret: &Return) -> GenerationResult {
        self.state.function_returned = true;
        let Return { expr } = ret;
        if let Some(expr) = expr {
            self.generate(expr)?;
        } else {
            self.emit_code(Opcode::Null);
        }
        self.emit_code(Opcode::Return);
        Ok(())
    }
}
