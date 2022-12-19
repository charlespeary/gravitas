use crate::{
    gravitas_std::{BuiltInFunction, FnArgs},
    MachineResult, OperationResult, RuntimeErrorCause, RuntimeValue, VM,
};
use bytecode::{
    callables::{Class, Function},
    chunk::Chunk,
};
use common::ProgramText;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct CallFrame {
    pub(crate) stack_start: usize,
    pub(crate) chunk: Chunk,
    pub(crate) name: ProgramText,
}

#[derive(Debug, Clone)]
pub enum Callable {
    Function(Function),
    Class(Class),
    BuiltInFunction(BuiltInFunction),
}

#[derive(Debug, Clone)]
pub struct ObjectInstance {
    pub class: Class,
    pub properties: HashMap<ProgramText, RuntimeValue>,
}

impl VM {
    fn get_args(&mut self, arity: usize) -> MachineResult<FnArgs> {
        let mut args = vec![];
        for _ in 0..arity {
            args.push(self.pop_operand()?);
        }

        Ok(args)
    }

    fn function_call(&mut self, function: Function) -> OperationResult {
        let frame = CallFrame {
            stack_start: self.ip + 1,
            chunk: function.chunk,
            name: function.name,
        };

        println!("calling: {}", frame.name);

        self.call_stack.push(frame);
        Ok(())
    }

    fn class_call(&mut self, class: Class) -> OperationResult {
        self.function_call(class.constructor)?;
        Ok(())
    }

    fn built_in_function_call(&mut self, built_in_function: BuiltInFunction) -> OperationResult {
        let BuiltInFunction {
            arity,
            fn_body,
            name,
        } = built_in_function;
        let args = self.get_args(arity)?;
        let result = fn_body(args, self);
        self.operands.push(result);
        Ok(())
    }

    pub(crate) fn op_call(&mut self) -> OperationResult {
        let callee = match self.pop_operand()? {
            RuntimeValue::Callable(callable) => callable,
            _ => return self.error(RuntimeErrorCause::NotCallable),
        };

        match callee {
            Callable::Function(function) => self.function_call(function),
            Callable::Class(class) => self.class_call(class),
            Callable::BuiltInFunction(built_in_fn) => self.built_in_function_call(built_in_fn),
        }?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use bytecode::{callables::Function, chunk::Constant, Opcode};
    use common::MAIN_FUNCTION_NAME;

    use crate::{test::new_vm, Chunk, OperationResult, VM};

    #[test]
    fn grow_callstack() {
        let function = Function {
            arity: 0,
            chunk: Chunk::default(),
            name: "foo".to_owned(),
        };

        let mut vm = new_vm(Chunk::new(
            vec![Opcode::Constant(0), Opcode::Call],
            vec![Constant::Function(function)],
        ));
    }

    #[test]
    fn change_callframe() -> OperationResult {
        let function = Function {
            arity: 0,
            chunk: Chunk::default(),
            name: "my_func".to_owned(),
        };

        let mut vm = VM::new(Chunk::new(
            vec![Opcode::Constant(0), Opcode::Call],
            vec![Constant::Function(function)],
        ));

        // we start with the global callframe which name is "main"
        let main_fn = vm.current_frame().name.clone();
        assert_eq!(&main_fn, MAIN_FUNCTION_NAME);
        // push the constant onto the stack
        vm.tick()?;
        // call the function
        vm.tick()?;
        // now the function's name should be equal to "my_func"
        let my_func = vm.current_frame().name.clone();
        assert_eq!(my_func, "my_func");

        Ok(())
    }
}
