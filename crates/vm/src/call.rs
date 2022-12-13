use crate::{OperationResult, RuntimeErrorCause, RuntimeValue, VM};
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
}

#[derive(Debug, Clone)]
pub struct ObjectInstance {
    class: Class,
    pub properties: HashMap<ProgramText, RuntimeValue>,
}

impl VM {
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

    pub(crate) fn op_call(&mut self) -> OperationResult {
        let callee = match self.pop_operand()? {
            RuntimeValue::Callable(callable) => callable,
            _ => return self.error(RuntimeErrorCause::NotCallable),
        };

        match callee {
            Callable::Function(function) => self.function_call(function),
            Callable::Class(class) => self.class_call(class),
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
