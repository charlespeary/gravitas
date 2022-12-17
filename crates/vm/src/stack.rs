use bytecode::MemoryAddress;
use common::Number;

use crate::{runtime_error::RuntimeErrorCause, MachineResult, RuntimeValue, VM};

impl VM {
    pub(crate) fn pop_number(&mut self) -> MachineResult<Number> {
        match self.pop_operand()? {
            RuntimeValue::Number(num) => Ok(num),
            _ => return self.error(RuntimeErrorCause::ExpectedNumber),
        }
    }

    pub(crate) fn pop_address(&mut self) -> MachineResult<MemoryAddress> {
        match self.pop_operand()? {
            RuntimeValue::MemoryAddress(address) => Ok(address),
            _ => return self.error(RuntimeErrorCause::ExpectedAddress),
        }
    }

    pub(crate) fn pop_operand(&mut self) -> MachineResult<RuntimeValue> {
        match self.operands.pop() {
            Some(value) => Ok(value),
            None => self.error(RuntimeErrorCause::PoppedFromEmptyStack),
        }
    }

    pub(crate) fn pop_two_operands(&mut self) -> MachineResult<(RuntimeValue, RuntimeValue)> {
        let b = self.pop_operand()?;
        let a = self.pop_operand()?;
        Ok((a, b))
    }
}

#[cfg(test)]
mod test {

    use bytecode::chunk::Chunk;

    use crate::{runtime_value::RuntimeValue, test::new_vm};

    #[test]
    fn pop_operand() {
        let mut vm = new_vm(Chunk::default());
        vm.operands = vec![
            RuntimeValue::Number(10.0),
            RuntimeValue::String("foo".to_owned()),
            RuntimeValue::Bool(false),
            RuntimeValue::Bool(true),
        ];
        assert!(vm
            .pop_operand()
            .unwrap()
            .eq(&RuntimeValue::Bool(true), &mut vm)
            .unwrap());

        assert!(vm
            .pop_operand()
            .unwrap()
            .eq(&RuntimeValue::Bool(false), &mut vm)
            .unwrap());

        assert!(vm
            .pop_operand()
            .unwrap()
            .eq(&RuntimeValue::String("foo".to_owned()), &mut vm)
            .unwrap());

        assert!(vm
            .pop_operand()
            .unwrap()
            .eq(&RuntimeValue::Number(10.0), &mut vm)
            .unwrap());
    }
}
