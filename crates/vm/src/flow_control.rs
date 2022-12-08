use crate::{runtime_value::RuntimeValue, OperationResult, VM};

impl VM {
    pub(crate) fn op_jif(&mut self) -> OperationResult {
        let distance = self.pop_number()?;
        let condition = self.pop_operand()?;

        if condition.eq(&RuntimeValue::Bool(false), self)? {
            self.move_pointer(distance as isize)?;
        }

        Ok(())
    }

    pub(crate) fn op_jp(&mut self) -> OperationResult {
        let distance = self.pop_number()?;
        self.move_pointer(distance as isize)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use bytecode::{
        chunk::{Chunk, Constant},
        Opcode,
    };

    use crate::{
        runtime_error::RuntimeErrorCause, runtime_value::RuntimeValue, test::new_vm,
        OperationResult,
    };

    #[test]
    fn op_jif() -> OperationResult {
        let code = Chunk::new(
            vec![
                Opcode::Constant(0),
                Opcode::Constant(1),
                Opcode::Constant(2),
                Opcode::Jif,
            ],
            vec![
                Constant::Number(127.0),
                Constant::Bool(false),
                Constant::Number(3.0),
            ],
        );
        let mut vm = new_vm(code);
        assert_eq!(vm.ip, 0);
        assert!(vm.run()?.eq(&RuntimeValue::Number(127.0), &mut vm)?);
        // opcodes advance the pointer to 0, 1, 2, and 3 and then we have a jump that advances by another 3 so from 3 to 6
        assert_eq!(vm.ip, 6);
        Ok(())
    }

    #[test]
    fn op_jp_forwards() -> OperationResult {
        let code = Chunk::new(
            vec![Opcode::Constant(0), Opcode::Constant(1), Opcode::Jp],
            vec![Constant::Number(127.0), Constant::Number(10.0)],
        );

        let mut vm = new_vm(code);
        assert_eq!(vm.ip, 0);
        // opcodes advance the pointer to 0, 1, and 2 and then we have a jump that advances by another 10 so 12
        assert!(vm.run()?.eq(&RuntimeValue::Number(127.0), &mut vm)?);
        assert_eq!(vm.ip, 12);

        Ok(())
    }

    #[test]
    fn op_jp_backwards() -> OperationResult {
        let code = Chunk::new(
            vec![Opcode::Constant(0), Opcode::Constant(1), Opcode::Jp],
            vec![Constant::Number(127.0), Constant::Number(-3.0)],
        );

        let mut vm = new_vm(code);
        assert_eq!(vm.ip, 0);
        // opcodes advance the pointer to 0, 1, and 2 and then we have a jump that retreats by 3 so -1
        // and that will cause a stack overflow
        // If we'd like to just test it to come back to a normal value then it would cause an infinite loop
        // therefore we have to crash the VM in order to check if its doing its job correctly
        assert_eq!(
            vm.run().unwrap_err().cause,
            RuntimeErrorCause::StackOverflow
        );

        Ok(())
    }
}
