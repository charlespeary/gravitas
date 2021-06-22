use common::Number;

use crate::{runtime_error::RuntimeErrorCause, MachineResult, RuntimeValue, VM};

impl VM {
    pub(crate) fn pop_operand(&mut self) -> MachineResult<RuntimeValue> {
        match self.operands.pop() {
            Some(value) => Ok(value),
            None => self.error(RuntimeErrorCause::PoppedFromEmptyStack),
        }
    }

    pub(crate) fn pop_two_operands(&mut self) -> MachineResult<(RuntimeValue, RuntimeValue)> {
        let a = self.pop_operand()?;
        let b = self.pop_operand()?;
        Ok((a, b))
    }

    pub(crate) fn pop_number(&mut self) -> MachineResult<Number> {
        let operand = self.pop_operand()?;

        match operand {
            RuntimeValue::Number(number) => Ok(number),
            _ => self.error(RuntimeErrorCause::ExpectedNumber),
        }
    }

    pub(crate) fn pop_bool(&mut self) -> MachineResult<bool> {
        let operand = self.pop_operand()?;

        match operand {
            RuntimeValue::Bool(bool) => Ok(bool),
            _ => self.error(RuntimeErrorCause::ExpectedBool),
        }
    }
}

#[cfg(test)]
mod test {

    use bytecode::chunk::Chunk;
    use lasso::Spur;

    use crate::{runtime_error::RuntimeErrorCause, runtime_value::RuntimeValue, test::new_vm};

    #[test]
    fn pop_operand() {
        let mut vm = new_vm(Chunk::default());
        vm.operands = vec![
            RuntimeValue::Number(10.0),
            RuntimeValue::String(Spur::default()),
            RuntimeValue::Bool(false),
            RuntimeValue::Bool(true),
        ];
        assert_eq!(vm.pop_operand().unwrap(), RuntimeValue::Bool(true));
        assert_eq!(vm.pop_operand().unwrap(), RuntimeValue::Bool(false));
        assert_eq!(
            vm.pop_operand().unwrap(),
            RuntimeValue::String(Spur::default())
        );
        assert_eq!(vm.pop_operand().unwrap(), RuntimeValue::Number(10.0));
    }

    #[test]
    fn pop_number() {
        let mut vm = new_vm(Chunk::default());
        vm.operands = vec![RuntimeValue::Number(10.0)];

        assert!(vm.pop_number().unwrap().is_normal());

        let mut vm = new_vm(Chunk::default());
        vm.operands = vec![RuntimeValue::Bool(false)];

        assert_eq!(
            vm.pop_number().unwrap_err().cause,
            RuntimeErrorCause::ExpectedNumber
        );
    }
}
