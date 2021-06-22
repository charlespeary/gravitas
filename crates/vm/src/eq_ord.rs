use crate::{
    runtime_error::RuntimeErrorCause, runtime_value::RuntimeValue, MachineResult, OperationResult,
    VM,
};

impl RuntimeValue {
    pub(crate) fn eq(&self, other: &RuntimeValue, vm: &mut VM) -> MachineResult<bool> {
        Ok(match (self, other) {
            (RuntimeValue::Number(a), RuntimeValue::Number(b)) => a == b,
            (RuntimeValue::String(a), RuntimeValue::String(b)) => a == b,
            (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => a == b,
            _ => return vm.error(RuntimeErrorCause::MismatchedTypes),
        })
    }
}

impl VM {
    pub(crate) fn op_eq(&mut self) -> OperationResult {
        let a = self.pop_operand()?;
        let b = self.pop_operand()?;
        let result = a.eq(&b, self)?;

        self.operands.push(RuntimeValue::Bool(result));
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use bytecode::{chunk::Constant, Opcode};
    use lasso::Spur;

    use crate::{
        runtime_error::RuntimeErrorCause,
        runtime_value::RuntimeValue,
        test::{create_failable_two_operand_assertion, create_two_operand_assertion},
    };

    #[test]
    fn op_eq() {
        let assert = create_two_operand_assertion(Opcode::Eq);

        let assert_numbers = |a: f64, b: f64| {
            let a = Constant::Number(a);
            let b = Constant::Number(b);

            // bidirectional equality
            // a == b
            assert(b, a, RuntimeValue::Bool(a == b));

            // b == a
            assert(a, b, RuntimeValue::Bool(b == a));
        };

        assert_numbers(0.0, 0.0);
        assert_numbers(10.0, 10.0);
        assert_numbers(-10.0, -10.0);
        assert_numbers(std::f64::MIN, std::f64::MIN);
        assert_numbers(std::f64::MAX, std::f64::MAX);
    }

    #[test]
    fn mismatched_types() {
        let assert_op_eq_err = |a, b| {
            let assert = create_failable_two_operand_assertion(Opcode::Eq);
            assert(a, b, RuntimeErrorCause::MismatchedTypes);
        };

        // can't compare numbers with strings
        assert_op_eq_err(Constant::String(Spur::default()), Constant::Number(10.0));
        // can't compare numbers with booleans
        assert_op_eq_err(Constant::Bool(true), Constant::Number(10.0));
        // can't compare strings with booleans
        assert_op_eq_err(Constant::Bool(true), Constant::String(Spur::default()));
    }
}
