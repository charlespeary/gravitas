use std::collections::HashMap;

use bytecode::{
    callables::{Class, Function},
    chunk::Chunk,
};
use common::Symbol;

use crate::{OperationResult, RuntimeErrorCause, RuntimeValue, VM};

#[derive(Debug, Clone)]
pub(crate) struct CallFrame {
    pub(crate) stack_start: usize,
    pub(crate) chunk: Chunk,
    pub(crate) name: Symbol,
}

#[derive(Debug, Clone)]
pub enum Callable {
    Function(Function),
    Class(Class),
}

#[derive(Debug, Clone)]
pub struct ObjectInstance {
    class: Class,
    pub properties: HashMap<Symbol, RuntimeValue>,
}

impl VM {
    fn function_call(&mut self, function: Function) -> OperationResult {
        let frame = CallFrame {
            stack_start: self.ip + 1,
            chunk: function.chunk,
            name: function.name,
        };

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
    use common::Symbol;
    use lasso::{Key, Rodeo};

    use crate::{test::new_vm, Chunk, OperationResult, VM};

    #[test]
    fn grow_callstack() {
        let function = Function {
            arity: 0,
            chunk: Chunk::default(),
            name: Symbol::default(),
        };

        let mut vm = new_vm(Chunk::new(
            vec![Opcode::Constant(0), Opcode::Call],
            vec![Constant::Function(function)],
        ));
    }

    #[test]
    fn change_callframe() -> OperationResult {
        let mut symbols = Rodeo::new();

        let global_func_name = "global";
        symbols.get_or_intern(global_func_name);

        let func_name = "my_function";

        let function = Function {
            arity: 0,
            chunk: Chunk::default(),
            name: symbols.get_or_intern(func_name),
        };

        let mut vm = VM::new(
            symbols.into_reader(),
            Chunk::new(
                vec![Opcode::Constant(0), Opcode::Call],
                vec![Constant::Function(function)],
            ),
        );

        // we start with the global callframe which name "global"
        // is always the first intern
        let name_symbol = vm.current_frame().name;
        assert_eq!(vm.symbols.resolve(&name_symbol), global_func_name);
        // push the constant onto the stack
        vm.tick()?;
        // call the function
        vm.tick()?;
        // now the function's name should be equal to "my_func"
        let name_symbol = vm.current_frame().name;
        assert_eq!(vm.symbols.resolve(&name_symbol), func_name);

        Ok(())
    }
}
