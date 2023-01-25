use crate::{
    gc::{HeapObject, HeapPointer},
    gravitas_std::{BuiltInFunction, FnArgs},
    MachineResult, RuntimeErrorCause, RuntimeValue, VM,
};
use common::ProgramText;

#[derive(Debug, Clone)]
pub(crate) struct CallFrame {
    pub(crate) stack_start: usize,
    pub(crate) name: ProgramText,
    pub(crate) return_ip: usize,
    pub(crate) closure_ptr: HeapPointer,
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
        self.debug(format!(
            "[CALL_STACK][NEW FRAME][NAME={}][RETURN_IP={}][STACK_START={}]",
            call_frame.name, call_frame.return_ip, call_frame.stack_start
        ));

        self.call_stack.push(call_frame);
    }

    pub(crate) fn remove_call_frame(&mut self) {
        let call_frame = self
            .call_stack
            .pop()
            .expect("Tried to remove the global call frame.");

        self.debug(format!(
            "[CALL_STACK][REMOVE FRAME][NAME={}][RETURN_IP={}][STACK_START={}]",
            call_frame.name, call_frame.return_ip, call_frame.stack_start
        ));

        self.ip = call_frame.return_ip;
        self.operands.truncate(call_frame.stack_start);
    }

    fn closure_call(&mut self, closure_ptr: HeapPointer) -> CallOperation {
        let closure = self.gc.deref(closure_ptr).as_closure();
        let function_ptr = closure.function_ptr;

        let (arity, name) = {
            let function = self.deref_global(function_ptr).as_function();

            (function.arity, function.name.clone())
        };

        self.debug(format!("[VM][CALL][FUNCTION][NAME={}]", &name));

        let recursion_handler = RuntimeValue::HeapPointer(closure_ptr);
        self.push_operand(recursion_handler);

        let frame = CallFrame {
            // -1 because we also count function pushed onto the stack
            // for recursion purposes
            stack_start: self.operands.len() - arity - 1,
            name,
            closure_ptr,
            return_ip: self.ip,
        };

        self.add_call_frame(frame);

        Ok(CallType::EnterFnBody)
    }

    fn bound_method_call(&mut self, method_ptr: HeapPointer) -> CallOperation {
        let bound_method = self.gc.deref(method_ptr).as_bound_method();

        let (arity, name) = {
            let function = self.deref_global(bound_method.method_ptr).as_function();

            (function.arity, function.name.clone())
        };

        self.push_operand(RuntimeValue::HeapPointer(bound_method.receiver));

        let frame = CallFrame {
            // -1 because we also count function pushed onto the stack
            // for recursion purposes
            stack_start: self.operands.len() - arity - 1,
            name,
            closure_ptr: method_ptr,
            return_ip: self.ip,
        };

        self.add_call_frame(frame);

        Ok(CallType::EnterFnBody)
    }

    // fn new_obj(&mut self, class_ptr: GlobalPointer) -> HeapPointer {
    //     let constructor_ptr = self.globals.get(class_ptr).unwrap().as_class().constructor;
    //     let instance = ObjectInstance {
    //         class_ptr,
    //         properties: HashMap::new(),
    //         constructor_ptr,
    //     };

    //     let instance_ptr = self.gc.allocate(HeapObject::Object(instance));
    //     instance_ptr
    // }

    fn built_in_function_call(&mut self, built_in_function: BuiltInFunction) -> CallOperation {
        let BuiltInFunction {
            arity,
            fn_body,
            name,
        } = built_in_function;

        self.debug(format!("[VM][CALL][BUILT IN][NAME={}]", &name));

        let args = self.get_args(arity)?;
        let result = fn_body(args, self);
        self.push_operand(result);
        Ok(CallType::InlineFn)
    }

    pub(crate) fn op_call(&mut self) -> CallOperation {
        let callee = self.pop_operand()?;

        match callee {
            // RuntimeValue::GlobalPointer(global_ptr) => self.class_call(global_ptr),
            RuntimeValue::HeapPointer(heap_ptr) => {
                let result = match self.gc.deref(heap_ptr) {
                    HeapObject::Closure(_) => self.closure_call(heap_ptr),
                    HeapObject::BoundMethod(_) => self.bound_method_call(heap_ptr),
                    _ => unreachable!(),
                };

                result
            }
            _ => self.error(RuntimeErrorCause::NotCallable),
        }
    }
}

#[cfg(test)]
mod test {
    use bytecode::{callables::Function, chunk::Constant, Opcode};
    use common::MAIN_FUNCTION_NAME;

    use crate::{test::main_fn, Chunk, OperationResult, VM};

    #[test]
    fn grow_callstack() {
        let function = Function {
            arity: 0,
            chunk: Chunk::default(),
            name: "foo".to_owned(),
        };

        let code = main_fn(Chunk::new(
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

        let mut code = main_fn(Chunk::new(
            vec![Opcode::Constant(0), Opcode::Call],
            vec![Constant::Function(function)],
        ));

        let mut vm = VM::new();

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
