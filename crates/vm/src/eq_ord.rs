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
            (RuntimeValue::ObjectInstance(a), RuntimeValue::ObjectInstance(b)) => {
                for (a_name, a_value) in &a.properties {
                    for (b_name, b_value) in &b.properties {
                        let equal_name = a_name == b_name;
                        let equal_value = a_value.eq(b_value, vm)?;

                        if !equal_name && !equal_value {
                            return Ok(false);
                        }
                    }
                }
                true
            }
            _ => false,
        })
    }

    #[allow(clippy::float_cmp)]
    pub(crate) fn ne(self, other: RuntimeValue, vm: &mut VM) -> MachineResult<bool> {
        self.eq(&other, vm).map(|bool| !bool)
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

    pub(crate) fn to_bool(self, vm: &mut VM) -> MachineResult<bool> {
        Ok(match self {
            RuntimeValue::Number(Number) => true,
            RuntimeValue::String(ProgramText) => true,
            RuntimeValue::Bool(bool) => bool,
            RuntimeValue::ObjectInstance(ObjectInstance) => true,
            RuntimeValue::Null => false,
            _ => unreachable!(),
        })
    }
}

impl VM {
    pub(crate) fn op_eq(&mut self) -> OperationResult {
        let (a, b) = self.pop_two_operands()?;
        let result = a.eq(&b, self)?;

        self.push_operand(RuntimeValue::Bool(result));
        Ok(())
    }

    pub(crate) fn op_ne(&mut self) -> OperationResult {
        let (a, b) = self.pop_two_operands()?;
        let result = a.ne(b, self)?;

        self.push_operand(RuntimeValue::Bool(result));
        Ok(())
    }

    pub(crate) fn op_gt(&mut self) -> OperationResult {
        let (a, b) = self.pop_two_operands()?;
        let result = a.gt(b, self)?;

        self.push_operand(RuntimeValue::Bool(result));
        Ok(())
    }

    pub(crate) fn op_ge(&mut self) -> OperationResult {
        let (a, b) = self.pop_two_operands()?;
        let result = a.ge(b, self)?;

        self.push_operand(RuntimeValue::Bool(result));
        Ok(())
    }

    pub(crate) fn op_lt(&mut self) -> OperationResult {
        let (a, b) = self.pop_two_operands()?;
        let result = a.lt(b, self)?;
        self.push_operand(RuntimeValue::Bool(result));
        Ok(())
    }

    pub(crate) fn op_le(&mut self) -> OperationResult {
        let (a, b) = self.pop_two_operands()?;
        let result = a.le(b, self)?;

        self.push_operand(RuntimeValue::Bool(result));
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
        runtime_error::RuntimeErrorCause,
        runtime_value::RuntimeValue,
        test::{
            assert_program, create_failable_two_operand_assertion, create_two_operand_assertion,
            dummy_class,
        },
    };

    #[test]
    fn op_eq() {
        let assert = create_two_operand_assertion(Opcode::Eq);

        let assert_numbers = |a: f64, b: f64| {
            let a = Constant::Number(a);
            let b = Constant::Number(b);

            let aeb = a == b;
            assert(b, a, RuntimeValue::Bool(aeb));
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
            Constant::String("foo".to_owned()),
            Constant::String("foo".to_owned()),
            RuntimeValue::Bool(false),
        );

        assert(
            Constant::String("foo".to_owned()),
            Constant::String("bar".to_owned()),
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
    fn op_gt() {
        let assert = create_two_operand_assertion(Opcode::Gt);

        assert(
            Constant::Number(0.0),
            Constant::Number(5.0),
            RuntimeValue::Bool(false),
        );

        assert(
            Constant::Number(10.0),
            Constant::Number(5.0),
            RuntimeValue::Bool(true),
        );

        assert(
            Constant::Number(5.0),
            Constant::Number(5.0),
            RuntimeValue::Bool(false),
        );
    }

    #[test]
    fn op_ge() {
        let assert = create_two_operand_assertion(Opcode::Ge);

        assert(
            Constant::Number(0.0),
            Constant::Number(5.0),
            RuntimeValue::Bool(false),
        );

        assert(
            Constant::Number(10.0),
            Constant::Number(5.0),
            RuntimeValue::Bool(true),
        );

        assert(
            Constant::Number(5.0),
            Constant::Number(5.0),
            RuntimeValue::Bool(true),
        );
    }

    #[test]
    fn op_lt() {
        let assert = create_two_operand_assertion(Opcode::Lt);

        assert(
            Constant::Number(10.0),
            Constant::Number(0.0),
            RuntimeValue::Bool(false),
        );

        assert(
            Constant::Number(10.0),
            Constant::Number(10.0),
            RuntimeValue::Bool(false),
        );

        assert(
            Constant::Number(5.0),
            Constant::Number(10.0),
            RuntimeValue::Bool(true),
        );
    }

    #[test]
    fn op_le() {
        let assert = create_two_operand_assertion(Opcode::Le);

        assert(
            Constant::Number(10.0),
            Constant::Number(0.0),
            RuntimeValue::Bool(false),
        );

        assert(
            Constant::Number(10.0),
            Constant::Number(10.0),
            RuntimeValue::Bool(true),
        );

        assert(
            Constant::Number(5.0),
            Constant::Number(10.0),
            RuntimeValue::Bool(true),
        );
    }

    #[test]
    fn mismatched_types() {
        let assert_err = |opcode, a, b| {
            let assert = create_failable_two_operand_assertion(opcode);
            assert(a, b, RuntimeErrorCause::MismatchedTypes);
        };

        let number_only_operations = vec![
            Opcode::Add,
            Opcode::Sub,
            Opcode::Mul,
            Opcode::Div,
            Opcode::Pow,
            Opcode::Mod,
            Opcode::Lt,
            Opcode::Le,
            Opcode::Gt,
            Opcode::Ge,
        ];

        for opcode in &number_only_operations {
            //  numbers with strings
            assert_err(
                *opcode,
                Constant::String("foo".to_owned()),
                Constant::Number(10.0),
            );

            //  numbers with booleans
            assert_err(*opcode, Constant::Bool(true), Constant::Number(10.0));

            //  strings with booleans
            assert_err(
                *opcode,
                Constant::Bool(true),
                Constant::String("foo".to_owned()),
            );
        }
    }

    #[test]
    fn object_comparison() {
        // TODO: implement when OOP is ready
        // let code = Chunk::new(
        //     vec![
        //         Opcode::Eq,
        //         Opcode::Call,
        //         Opcode::Call,
        //         Opcode::Constant(0),
        //         Opcode::Constant(0),
        //     ],
        //     vec![Constant::Class(dummy_class())],
        // );

        // assert_program(code, RuntimeValue::Bool(true));
    }
}
