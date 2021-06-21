use crate::{runtime_value::RuntimeValue, OperationResult, VM};

impl VM {
    pub(crate) fn op_add(&mut self) -> OperationResult {
        let a = self.pop_number()?;
        let b = self.pop_number()?;
        self.operands.push(RuntimeValue::Number(a + b));
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use bytecode::{
        chunk::{Chunk, Constant},
        Opcode,
    };

    use crate::{runtime_error::RuntimeErrorCause, runtime_value::RuntimeValue, test::new_vm};

    #[test]
    fn op_add() {
        let mut vm = new_vm(Chunk::new(
            vec![Opcode::Constant(0), Opcode::Constant(1), Opcode::Add],
            vec![Constant::Number(3.0), Constant::Number(4.0)],
        ));

        assert_eq!(vm.run().unwrap(), RuntimeValue::Number(7.0));
    }

    #[test]
    fn op_add_fail() {
        let mut vm = new_vm(Chunk::new(
            vec![Opcode::Constant(0), Opcode::Constant(1), Opcode::Add],
            vec![Constant::Bool(false), Constant::Bool(true)],
        ));

        assert_eq!(
            vm.run().unwrap_err().cause,
            RuntimeErrorCause::ExpectedNumber
        );
    }
}
