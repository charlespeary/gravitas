use crate::{runtime_error::RuntimeErrorCause, OperationResult, VM};

impl VM {
    pub(crate) fn op_pop(&mut self) -> OperationResult {
        let n = self.pop_number()?;

        for _ in 0..n as usize {
            self.pop_operand()?;
        }

        Ok(())
    }

    pub(crate) fn op_asg(&mut self) -> OperationResult {
        unimplemented!()
    }

    pub(crate) fn op_get(&mut self) -> OperationResult {
        let address = self.pop_number()?;
        let stack_start = self.current_frame().stack_start;
        match self.operands.get(stack_start + address as usize).cloned() {
            Some(value) => {
                self.operands.push(value);
                Ok(())
            }
            None => self.error(RuntimeErrorCause::StackOverflow),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{runtime_value::RuntimeValue, test::new_vm, OperationResult};
    use bytecode::{
        chunk::{Chunk, Constant},
        Opcode,
    };

    #[test]
    fn op_pop() -> OperationResult {
        let mut vm = new_vm(Chunk::new(
            vec![
                Opcode::Constant(0),
                Opcode::Constant(1),
                Opcode::Constant(2),
                Opcode::Constant(3),
                Opcode::Pop,
            ],
            vec![
                Constant::Bool(true),
                Constant::Bool(true),
                Constant::Bool(true),
                Constant::Number(3.0),
            ],
        ));

        // let's push the constants onto the stack
        vm.tick()?;
        vm.tick()?;
        vm.tick()?;
        vm.tick()?;

        assert_eq!(vm.operands.len(), 4);

        vm.tick()?;

        // Op::Pop will pop one operand and this operand will tell it to pop 3 values from the operands stack
        // so after the operation finishes the operands stack length will be equal to 0
        assert_eq!(vm.operands.len(), 0);

        Ok(())
    }

    #[test]
    fn op_get() -> OperationResult {
        let mut vm = new_vm(Chunk::new(
            vec![Opcode::Constant(0), Opcode::Constant(1), Opcode::Get],
            vec![Constant::Bool(true), Constant::Number(0.0)],
        ));

        // push the constants onto the stack
        vm.tick()?;
        vm.tick()?;
        // execute get
        vm.tick()?;
        // only Constant::Bool(true) should be present on the stack after it got pushed back there
        let leftover_value = vm.operands[0].clone();

        assert!(leftover_value
            .eq(RuntimeValue::Bool(true), &mut vm)
            .unwrap());

        Ok(())
    }
}
