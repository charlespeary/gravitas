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
    pub(crate) return_ip: usize,
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

pub(crate) enum CallType {
    EnterFnBody,
    InlineFn,
}

pub(crate) type CallOperation = MachineResult<CallType>;

impl VM {
    fn get_args(&mut self, arity: usize) -> MachineResult<FnArgs> {
        let mut args = vec![];
        for _ in 0..arity {
            args.push(self.pop_operand()?);
        }

        Ok(args)
    }

    pub(crate) fn add_call_frame(&mut self, call_frame: CallFrame) {
        self.call_stack.push(call_frame);
    }

    pub(crate) fn remove_call_frame(&mut self) {
        let frame = self
            .call_stack
            .pop()
            .expect("Tried to remove the global call frame.");

        println!(
            "return_ip: {}, self.ip: {} stack_start: {}",
            frame.return_ip, self.ip, frame.stack_start
        );

        self.ip = frame.return_ip;
        // println!("stack bt: {:#?}", &self.operands);
        self.operands.truncate(frame.stack_start);
        // println!("stack at: {:#?}", &self.operands);
    }

    fn function_call(&mut self, function: Function) -> CallOperation {
        let recursion_handler = function.clone();
        self.operands.push(recursion_handler.into());

        let frame = CallFrame {
            // -1 because we also count function pushed onto the stack
            // for recursion purposes
            stack_start: self.operands.len() - function.arity - 1,
            chunk: function.chunk,
            name: function.name,
            return_ip: self.ip,
        };

        self.add_call_frame(frame);

        Ok(CallType::EnterFnBody)
    }

    fn class_call(&mut self, class: Class) -> CallOperation {
        self.function_call(class.constructor)?;
        Ok(CallType::EnterFnBody)
    }

    fn built_in_function_call(&mut self, built_in_function: BuiltInFunction) -> CallOperation {
        let BuiltInFunction {
            arity,
            fn_body,
            name,
        } = built_in_function;
        let args = self.get_args(arity)?;
        let result = fn_body(args, self);
        self.operands.push(result);
        Ok(CallType::InlineFn)
    }

    pub(crate) fn op_call(&mut self) -> CallOperation {
        let callee = match self.pop_operand()? {
            RuntimeValue::Callable(callable) => callable,
            _ => return self.error(RuntimeErrorCause::NotCallable),
        };

        match callee {
            Callable::Function(function) => self.function_call(function),
            Callable::Class(class) => self.class_call(class),
            Callable::BuiltInFunction(built_in_fn) => self.built_in_function_call(built_in_fn),
        }
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
