use bytecode::MemoryAddress;

use crate::{
    gravitas_std::NATIVE_FUNCTIONS, runtime_error::RuntimeErrorCause, runtime_value::RuntimeValue,
    MachineResult, OperationResult, VM,
};

impl VM {
    pub(crate) fn op_pop(&mut self, amount: usize) -> OperationResult {
        for _ in 0..amount as usize {
            self.pop_operand()?;
        }

        Ok(())
    }

    pub(crate) fn assign_value(
        &mut self,
        value: RuntimeValue,
        address: MemoryAddress,
    ) -> OperationResult {
        let stack_start = self.current_frame().stack_start;

        self.debug(format!(
            "[STACK][ASSIGN][ADDRESS={}][VALUE={}]",
            &address, &value
        ));

        match address {
            MemoryAddress::Local(local_address) => {
                self.operands[stack_start + local_address as usize] = value;
            }
            MemoryAddress::Upvalue { index, is_ref } => {
                let current_closure_ptr = self
                    .call_stack
                    .last()
                    .map(|frame| frame.closure_ptr)
                    .unwrap();

                let closure = self.gc.deref(current_closure_ptr).as_closure();

                let mut upvalue_ptr = closure.upvalues.get(index).cloned().unwrap();

                if is_ref {
                    while let RuntimeValue::HeapPointer(new_upvalue_ptr) =
                        self.gc.deref(upvalue_ptr).as_value()
                    {
                        upvalue_ptr = *new_upvalue_ptr;
                    }
                }

                *self.gc.deref_mut(upvalue_ptr) = value.into();
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    pub(crate) fn op_asg(&mut self) -> OperationResult {
        let to_assign = self.pop_operand()?;
        let address = self.pop_address()?;
        self.assign_value(to_assign, address)?;

        Ok(())
    }

    pub(crate) fn get_local_variable(
        &mut self,
        local_address: usize,
    ) -> MachineResult<RuntimeValue> {
        let stack_start = self.current_frame().stack_start;
        let stack_address = stack_start + local_address as usize;

        match self.operands.get(stack_address).cloned() {
            Some(value) => {
                self.debug(format!(
                    "[STACK][GET_LOCAL_VARIABLE][ADDRESS={}][VALUE={}]",
                    stack_address, &value
                ));
                Ok(value)
            }
            None => self.error(RuntimeErrorCause::StackOverflow),
        }
    }

    pub(crate) fn get_upvalue(
        &mut self,
        upvalue_index: usize,
        is_ref: bool,
    ) -> MachineResult<RuntimeValue> {
        let current_closure_ptr = self
            .call_stack
            .last()
            .map(|frame| frame.closure_ptr)
            .unwrap();

        let closure = self.gc.deref(current_closure_ptr).as_closure();
        let upvalue_ptr = closure.upvalues.get(upvalue_index).cloned().unwrap();
        let mut upvalue = self.gc.deref(upvalue_ptr).as_value();

        if is_ref {
            while let RuntimeValue::HeapPointer(upvalue_ptr) = upvalue {
                upvalue = self.gc.deref(*upvalue_ptr).as_value();
            }
        }

        Ok(upvalue.clone())
    }

    pub(crate) fn get_variable(&mut self, address: MemoryAddress) -> MachineResult<RuntimeValue> {
        match address {
            MemoryAddress::Local(stack_address) => self.get_local_variable(stack_address),
            MemoryAddress::Upvalue { index, is_ref } => self.get_upvalue(index, is_ref),
            MemoryAddress::BuiltInFunction(built_in_function) => {
                Ok(RuntimeValue::NativeFunction(built_in_function))
            }
        }
    }

    pub(crate) fn op_get(&mut self) -> OperationResult {
        let address = self.pop_address()?;
        let value = self.get_variable(address)?;
        self.push_operand(value);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{runtime_value::RuntimeValue, test::main_fn, OperationResult, VM};
    use bytecode::{
        chunk::{Chunk, Constant},
        MemoryAddress, Opcode,
    };

    #[test]
    fn op_pop() -> OperationResult {
        let mut code = main_fn(Chunk::new(
            vec![
                Opcode::Constant(0),
                Opcode::Constant(1),
                Opcode::Constant(2),
                Opcode::Pop(3),
            ],
            vec![
                Constant::Bool(true),
                Constant::Bool(true),
                Constant::Bool(true),
            ],
        ));

        let mut vm = VM::new();

        // let's push the constants onto the stack
        vm.tick()?;
        vm.tick()?;
        vm.tick()?;

        assert_eq!(vm.operands.len(), 3);

        vm.tick()?;

        // Pop(3) will pop 3 operands from the stack
        // so after the operation finishes the operands stack length will be equal to 0
        assert_eq!(vm.operands.len(), 0);

        Ok(())
    }

    #[test]
    fn op_get() -> OperationResult {
        let mut vm = VM::new();
        let mut code = main_fn(Chunk::new(
            vec![Opcode::Constant(0), Opcode::Constant(1), Opcode::Get],
            vec![
                Constant::Bool(true),
                Constant::MemoryAddress(MemoryAddress::Local(0)),
            ],
        ));

        // push the constants onto the stack
        vm.tick()?;
        vm.tick()?;
        // execute get
        vm.tick()?;
        // only Constant::Bool(true) should be present on the stack after it got pushed back there
        let leftover_value = vm.operands[0].clone();

        assert!(leftover_value
            .eq(&RuntimeValue::Bool(true), &mut vm)
            .unwrap());

        Ok(())
    }

    #[test]
    fn op_asg() -> OperationResult {
        let mut vm = VM::new();
        let mut code = main_fn(Chunk::new(
            vec![
                Opcode::Constant(0),
                Opcode::Constant(1),
                Opcode::Constant(2),
                Opcode::Asg,
            ],
            vec![
                Constant::Number(127.0),
                Constant::MemoryAddress(MemoryAddress::Local(0)),
                Constant::Number(7.0),
            ],
        ));

        // push the constants onto the stack
        vm.tick()?;
        vm.tick()?;
        vm.tick()?;

        // the first operand on the stack is the initial value of the variable
        let first_operand = vm.operands[0].clone();
        assert!(first_operand
            .eq(&RuntimeValue::Number(127.0), &mut vm)
            .unwrap());

        // execute Opcode::Asg
        vm.tick()?;

        // but after the execution the first operand on the stack will change to the assigned value
        let assigned_value = vm.operands[0].clone();
        assert!(assigned_value
            .eq(&RuntimeValue::Number(7.0), &mut vm)
            .unwrap());

        Ok(())
    }
}
