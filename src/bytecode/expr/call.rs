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

#[cfg(test)]
mod test {
    use crate::{
        bytecode::{test::generate_bytecode, Address, Value},
        parser::{
            expr::{atom::Atom, call::Args, Identifier},
            Expr,
        },
    };

    use super::*;

    #[test]
    fn call() {
        let ast = Expr::Call(Call {
            args: Args(vec![
                Expr::Atom(Atom::Number(10.0)),
                Expr::Atom(Atom::Number(5.0)),
            ]),
            // It will evaluate to global identifier, because print is available in hashmap of globals
            caller: Box::new(Expr::Identifier(Identifier {
                value: String::from("print"),
                is_ref: false,
            })),
        });
        let (chunk, code) = generate_bytecode(ast);

        assert_eq!(
            code,
            vec![
                Opcode::Constant(0),
                Opcode::Constant(1),
                Opcode::Constant(2),
                Opcode::Get,
                Opcode::Call
            ]
        );

        // 10.0
        assert_eq!(*chunk.read_constant(0), Value::Number(10.0));
        // 5.0
        assert_eq!(*chunk.read_constant(1), Value::Number(5.0));
        // print global address
        assert_eq!(
            *chunk.read_constant(2),
            Value::Address(Address::Global(String::from("print")))
        );
    }
}
