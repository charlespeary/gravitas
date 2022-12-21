use bytecode::MemoryAddress;
use common::Number;
use prettytable::{Cell, Row, Table};

use crate::{runtime_error::RuntimeErrorCause, MachineResult, RuntimeValue, VM};

impl VM {
    fn debug_stack(&mut self) {
        let mut table = Table::new();

        table.add_row(row!["INDEX", "STACK VALUE"]);

        for (index, value) in self.operands.iter().enumerate() {
            table.add_row(row![index, value]);
        }

        self.debug("");
        self.debug(table.to_string());
    }

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

    pub(crate) fn push_operand(&mut self, operand: RuntimeValue) {
        self.debug(format!("[STACK][PUSH] {}", &operand));
        self.debug_stack();

        self.operands.push(operand);
    }

    pub(crate) fn pop_operand(&mut self) -> MachineResult<RuntimeValue> {
        let value = self.operands.pop();
        self.debug(format!("[STACK][POP] {:?}", &value));
        self.debug_stack();

        match value {
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

    use crate::{runtime_value::RuntimeValue, VM};

    #[test]
    fn pop_operand() {
        let mut vm = VM::new();
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
