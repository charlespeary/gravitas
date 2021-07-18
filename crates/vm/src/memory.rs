use crate::{runtime_error::RuntimeErrorCause, runtime_value::RuntimeValue, OperationResult, VM};

impl VM {
    pub(crate) fn op_pop(&mut self) -> OperationResult {
        match self.pop_operand()? {
            RuntimeValue::Number(n) => {
                for _ in 0..n as usize {
                    self.pop_operand()?;
                }
            }
            _ => return self.error(RuntimeErrorCause::ExpectedUsize),
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{test::new_vm, OperationResult};
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
}
