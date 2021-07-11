use crate::{
    runtime_error::RuntimeErrorCause, runtime_value::RuntimeValue, MachineResult, OperationResult,
    VM,
};

impl RuntimeValue {
    pub(crate) fn eq(self, other: RuntimeValue, vm: &mut VM) -> MachineResult<bool> {
        Ok(match (self, other) {
            (RuntimeValue::Number(a), RuntimeValue::Number(b)) => a == b,
            (RuntimeValue::String(a), RuntimeValue::String(b)) => a == b,
            (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => a == b,
            _ => return vm.error(RuntimeErrorCause::MismatchedTypes),
        })
    }

    #[allow(clippy::float_cmp)]
    pub(crate) fn ne(self, other: RuntimeValue, vm: &mut VM) -> MachineResult<bool> {
        Ok(match (self, other) {
            (RuntimeValue::Number(a), RuntimeValue::Number(b)) => a != b,
            (RuntimeValue::String(a), RuntimeValue::String(b)) => a != b,
            (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => a != b,
            _ => return vm.error(RuntimeErrorCause::MismatchedTypes),
        })
    }

    pub(crate) fn gt(self, other: RuntimeValue, vm: &mut VM) -> MachineResult<bool> {
        Ok(match (self, other) {
            (RuntimeValue::Number(a), RuntimeValue::Number(b)) => a > b,
            _ => return vm.error(RuntimeErrorCause::MismatchedTypes),
        })
    }

    pub(crate) fn ge(self, other: RuntimeValue, vm: &mut VM) -> MachineResult<bool> {
        Ok(match (self, other) {
            (RuntimeValue::Number(a), RuntimeValue::Number(b)) => a >= b,
            _ => return vm.error(RuntimeErrorCause::MismatchedTypes),
        })
    }

    pub(crate) fn lt(self, other: RuntimeValue, vm: &mut VM) -> MachineResult<bool> {
        Ok(match (self, other) {
            (RuntimeValue::Number(a), RuntimeValue::Number(b)) => a < b,
            _ => return vm.error(RuntimeErrorCause::MismatchedTypes),
        })
    }

    pub(crate) fn le(self, other: RuntimeValue, vm: &mut VM) -> MachineResult<bool> {
        Ok(match (self, other) {
            (RuntimeValue::Number(a), RuntimeValue::Number(b)) => a <= b,
            _ => return vm.error(RuntimeErrorCause::MismatchedTypes),
        })
    }
}

impl VM {
    pub(crate) fn op_eq(&mut self) -> OperationResult {
        let (a, b) = self.pop_two_operands()?;
        let result = a.eq(b, self)?;

        self.operands.push(RuntimeValue::Bool(result));
        Ok(())
    }

    pub(crate) fn op_ne(&mut self) -> OperationResult {
        let (a, b) = self.pop_two_operands()?;
        let result = a.ne(b, self)?;

        self.operands.push(RuntimeValue::Bool(result));
        Ok(())
    }

    pub(crate) fn op_gt(&mut self) -> OperationResult {
        let (a, b) = self.pop_two_operands()?;
        let result = a.gt(b, self)?;

        self.operands.push(RuntimeValue::Bool(result));
        Ok(())
    }

    pub(crate) fn op_ge(&mut self) -> OperationResult {
        let (a, b) = self.pop_two_operands()?;
        let result = a.ge(b, self)?;

        self.operands.push(RuntimeValue::Bool(result));
        Ok(())
    }

    pub(crate) fn op_lt(&mut self) -> OperationResult {
        let (a, b) = self.pop_two_operands()?;
        let result = a.lt(b, self)?;

        self.operands.push(RuntimeValue::Bool(result));
        Ok(())
    }

    pub(crate) fn op_le(&mut self) -> OperationResult {
        let (a, b) = self.pop_two_operands()?;
        let result = a.le(b, self)?;

        self.operands.push(RuntimeValue::Bool(result));
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use bytecode::{chunk::Constant, Opcode};
    use lasso::{Key, Spur};

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
    fn op_ne() {
        let assert = create_two_operand_assertion(Opcode::Ne);

        assert(
            Constant::Number(10.0),
            Constant::Number(10.0),
            RuntimeValue::Bool(false),
        );

        assert(
            Constant::Number(10.0),
            Constant::Number(15.0),
            RuntimeValue::Bool(true),
        );

        assert(
            Constant::String(Spur::try_from_usize(0).unwrap()),
            Constant::String(Spur::try_from_usize(0).unwrap()),
            RuntimeValue::Bool(false),
        );

        assert(
            Constant::String(Spur::try_from_usize(0).unwrap()),
            Constant::String(Spur::try_from_usize(1).unwrap()),
            RuntimeValue::Bool(true),
        );

        assert(
            Constant::Bool(true),
            Constant::Bool(false),
            RuntimeValue::Bool(true),
        );

        assert(
            Constant::Bool(true),
            Constant::Bool(true),
            RuntimeValue::Bool(false),
        );

        assert(
            Constant::Bool(false),
            Constant::Bool(false),
            RuntimeValue::Bool(false),
        );
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
