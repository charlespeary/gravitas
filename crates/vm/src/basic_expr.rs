use std::ops::Neg;

use bytecode::chunk::ConstantIndex;

use crate::{
    runtime_error::RuntimeErrorCause, runtime_value::RuntimeValue, MachineResult, OperationResult,
    VM,
};

impl RuntimeValue {
    pub(crate) fn add(self, other: RuntimeValue, vm: &mut VM) -> MachineResult<RuntimeValue> {
        match (self, other) {
            (RuntimeValue::Number(a), RuntimeValue::Number(b)) => Ok(RuntimeValue::Number(a + b)),
            _ => vm.error(RuntimeErrorCause::MismatchedTypes),
        }
    }

    // TODO: This pattern matching will be used in many places
    // It'd be nice to have it somewhere as a helper function
    pub(crate) fn sub(self, other: RuntimeValue, vm: &mut VM) -> MachineResult<RuntimeValue> {
        match (self, other) {
            (RuntimeValue::Number(a), RuntimeValue::Number(b)) => Ok(RuntimeValue::Number(a - b)),
            _ => vm.error(RuntimeErrorCause::MismatchedTypes),
        }
    }

    pub(crate) fn mul(self, other: RuntimeValue, vm: &mut VM) -> MachineResult<RuntimeValue> {
        match (self, other) {
            (RuntimeValue::Number(a), RuntimeValue::Number(b)) => Ok(RuntimeValue::Number(a * b)),
            _ => vm.error(RuntimeErrorCause::MismatchedTypes),
        }
    }

    pub(crate) fn div(self, other: RuntimeValue, vm: &mut VM) -> MachineResult<RuntimeValue> {
        match (self, other) {
            (RuntimeValue::Number(a), RuntimeValue::Number(b)) => Ok(RuntimeValue::Number(a / b)),
            _ => vm.error(RuntimeErrorCause::MismatchedTypes),
        }
    }

    pub(crate) fn modulo(self, other: RuntimeValue, vm: &mut VM) -> MachineResult<RuntimeValue> {
        match (self, other) {
            (RuntimeValue::Number(a), RuntimeValue::Number(b)) => Ok(RuntimeValue::Number(a % b)),
            _ => vm.error(RuntimeErrorCause::MismatchedTypes),
        }
    }

    pub(crate) fn pow(self, other: RuntimeValue, vm: &mut VM) -> MachineResult<RuntimeValue> {
        match (self, other) {
            (RuntimeValue::Number(a), RuntimeValue::Number(b)) => {
                Ok(RuntimeValue::Number(a.powf(b)))
            }
            _ => vm.error(RuntimeErrorCause::MismatchedTypes),
        }
    }

    pub(crate) fn and(self, other: RuntimeValue, vm: &mut VM) -> MachineResult<RuntimeValue> {
        match (self, other) {
            (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => Ok(RuntimeValue::Bool(a && b)),
            _ => vm.error(RuntimeErrorCause::MismatchedTypes),
        }
    }

    pub(crate) fn or(self, other: RuntimeValue, vm: &mut VM) -> MachineResult<RuntimeValue> {
        match (self, other) {
            (RuntimeValue::Bool(a), RuntimeValue::Bool(b)) => Ok(RuntimeValue::Bool(a || b)),
            _ => vm.error(RuntimeErrorCause::MismatchedTypes),
        }
    }

    pub(crate) fn not(self, vm: &mut VM) -> MachineResult<RuntimeValue> {
        match self {
            RuntimeValue::Bool(a) => Ok(RuntimeValue::Bool(!a)),
            _ => vm.error(RuntimeErrorCause::MismatchedTypes),
        }
    }

    pub(crate) fn neg(self, vm: &mut VM) -> MachineResult<RuntimeValue> {
        match self {
            RuntimeValue::Number(a) => Ok(RuntimeValue::Number(a.neg())),
            _ => vm.error(RuntimeErrorCause::MismatchedTypes),
        }
    }
}

impl VM {
    // Start of stuff that doesn't belong to any particular group

    pub(crate) fn op_constant(&mut self, index: ConstantIndex) -> OperationResult {
        let item = self.current_frame().chunk.read(index);
        let value = RuntimeValue::from(item);
        self.push_operand(value);
        Ok(())
    }

    // End of stuff that doesn't belong to any particular group

    // Start of unary expressions

    pub(crate) fn op_neg(&mut self) -> OperationResult {
        let a = self.pop_operand()?;
        let res = a.neg(self)?;
        self.push_operand(res);
        Ok(())
    }

    pub(crate) fn op_not(&mut self) -> OperationResult {
        let a = self.pop_operand()?;
        let res = a.not(self)?;
        self.push_operand(res);
        Ok(())
    }

    // End of unary expressions

    // Start of binary expressions
    pub(crate) fn op_add(&mut self) -> OperationResult {
        let (a, b) = self.pop_two_operands()?;
        let res = a.add(b, self)?;
        self.push_operand(res);
        Ok(())
    }

    pub(crate) fn op_sub(&mut self) -> OperationResult {
        let (a, b) = self.pop_two_operands()?;
        let res = a.sub(b, self)?;
        self.push_operand(res);
        Ok(())
    }

    pub(crate) fn op_mul(&mut self) -> OperationResult {
        let (a, b) = self.pop_two_operands()?;
        let res = a.mul(b, self)?;
        self.push_operand(res);
        Ok(())
    }

    pub(crate) fn op_div(&mut self) -> OperationResult {
        let (a, b) = self.pop_two_operands()?;
        let res = a.div(b, self)?;
        self.push_operand(res);
        Ok(())
    }

    pub(crate) fn op_mod(&mut self) -> OperationResult {
        let (a, b) = self.pop_two_operands()?;
        let res = a.modulo(b, self)?;
        self.push_operand(res);
        Ok(())
    }

    pub(crate) fn op_pow(&mut self) -> OperationResult {
        let (a, b) = self.pop_two_operands()?;
        let res = a.pow(b, self)?;
        self.push_operand(res);
        Ok(())
    }

    pub(crate) fn op_and(&mut self) -> OperationResult {
        let (a, b) = self.pop_two_operands()?;
        let res = a.and(b, self)?;
        self.push_operand(res);
        Ok(())
    }

    pub(crate) fn op_or(&mut self) -> OperationResult {
        let (a, b) = self.pop_two_operands()?;
        let res = a.or(b, self)?;
        self.push_operand(res);
        Ok(())
    }

    // End of binary expressions
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
        test::{assert_program, create_two_operand_assertion},
    };

    // Start of stuff that doesn't belong to any particular group

    #[test]
    fn op_constant() {
        fn assert_constant(constant: Constant) {
            assert_program(
                Chunk::new(vec![Opcode::Constant(0)], vec![constant.clone()]),
                RuntimeValue::from(constant),
            );
        }

        assert_constant(Constant::Bool(false));
        assert_constant(Constant::Bool(true));
        assert_constant(Constant::String("foo".to_owned()));
        assert_constant(Constant::Number(std::f64::MAX));
        assert_constant(Constant::Number(std::f64::MIN));
    }

    // End of stuff that doesn't belong to any particular group

    // Start of unary expressions

    #[test]
    fn op_neg() {
        // Accept only booleans
        let mut code = new_vm(Chunk::new(
            vec![Opcode::Constant(0), Opcode::Neg],
            vec![Constant::Bool(true)],
        ));

        assert_eq!(
            vm.run().unwrap_err().cause,
            RuntimeErrorCause::MismatchedTypes
        );

        let assert_neg = |a, e| {
            let mut vm = new_vm(Chunk::new(
                vec![Opcode::Constant(0), Opcode::Neg],
                vec![Constant::Number(a)],
            ));

            assert!(vm
                .run()
                .unwrap()
                .eq(&RuntimeValue::Number(e), &mut vm)
                .unwrap())
        };

        assert_neg(1.0, -1.0);
        assert_neg(-1.0, 1.0);
        assert_neg(0.0, 0.0);
        assert_neg(std::f64::MAX, std::f64::MIN);
        assert_neg(std::f64::MIN, std::f64::MAX);
    }

    #[test]
    fn op_not() {
        // Accept only booleans
        let mut vm = new_vm(Chunk::new(
            vec![Opcode::Constant(0), Opcode::Not],
            vec![Constant::Number(10.0)],
        ));

        assert_eq!(
            vm.run().unwrap_err().cause,
            RuntimeErrorCause::MismatchedTypes
        );

        let assert_not = |a, e| {
            let mut vm = new_vm(Chunk::new(
                vec![Opcode::Constant(0), Opcode::Not],
                vec![Constant::Bool(a)],
            ));

            assert!(vm
                .run()
                .unwrap()
                .eq(&RuntimeValue::Bool(e), &mut vm)
                .unwrap())
        };

        assert_not(false, true);
        assert_not(true, false);
    }

    // End of unary expressions

    // Start of binary expressions

    fn assert_arithmetic_op(opcode: Opcode) -> impl Fn(f64, f64, f64) {
        let assertion = create_two_operand_assertion(opcode);
        move |a: f64, b: f64, e: f64| {
            assertion(
                Constant::Number(a),
                Constant::Number(b),
                RuntimeValue::Number(e),
            )
        }
    }

    #[test]
    fn op_add() {
        let assert_add = assert_arithmetic_op(Opcode::Add);

        assert_add(-10.0, 10.0, 0.0);
        assert_add(10.0, 20.0, 30.0);
        assert_add(0.0, 0.0, 0.0);
        assert_add(std::f64::MAX, std::f64::MAX, std::f64::INFINITY);
        assert_add(std::f64::MIN, std::f64::MIN, std::f64::NEG_INFINITY);
    }

    #[test]
    fn op_expects_numbers() {
        let expect_numbers = |opcode| {
            let mut vm = new_vm(Chunk::new(
                vec![Opcode::Constant(0), Opcode::Constant(1), opcode],
                vec![Constant::Bool(false), Constant::Bool(true)],
            ));
            assert_eq!(
                vm.run().unwrap_err().cause,
                RuntimeErrorCause::MismatchedTypes
            );
        };

        expect_numbers(Opcode::Add);
        expect_numbers(Opcode::Sub);
        expect_numbers(Opcode::Mul);
        expect_numbers(Opcode::Div);
        expect_numbers(Opcode::Mod);
        expect_numbers(Opcode::Pow);
    }

    #[test]
    fn op_sub() {
        let assert_sub = assert_arithmetic_op(Opcode::Sub);

        // Expect 10.0 to be on top of the stack
        assert_sub(10.0, 0.0, 10.0);
        assert_sub(0.0, 10.0, -10.0);
        assert_sub(std::f64::MIN, std::f64::MIN, 0.0);
        assert_sub(std::f64::MAX, std::f64::MAX, 0.0);
        assert_sub(std::f64::MIN, -std::f64::MAX, 0.0);
    }

    #[test]
    fn op_mul() {
        let assert_mul = assert_arithmetic_op(Opcode::Mul);

        assert_mul(1.0, 1.0, 1.0);
        assert_mul(10.0, 10.0, 100.0);
        assert_mul(0.0, 0.0, 0.0);
        assert_mul(-1.0, -1.0, 1.0);
        assert_mul(std::f64::MAX, std::f64::MIN, std::f64::NEG_INFINITY);
        assert_mul(std::f64::MAX, std::f64::MAX, std::f64::INFINITY);
        assert_mul(std::f64::MIN, std::f64::MIN, std::f64::INFINITY);
    }

    #[test]
    fn op_div() {
        let mut vm = new_vm(Chunk::new(
            vec![Opcode::Constant(0), Opcode::Constant(1), Opcode::Div],
            vec![Constant::Number(0.0), Constant::Number(0.0)],
        ));

        if let RuntimeValue::Number(nan) = vm.run().unwrap() {
            assert!(nan.is_nan());
        } else {
            panic!("Expected NaN");
        }

        let assert_div = assert_arithmetic_op(Opcode::Div);
        assert_div(std::f64::MAX, std::f64::MAX, 1.0);
        assert_div(std::f64::MIN, std::f64::MIN, 1.0);
        assert_div(10.0, 1.0, 10.0);
        assert_div(-1.0, -1.0, 1.0);
    }

    #[test]
    fn op_mod() {
        let assert_mod = assert_arithmetic_op(Opcode::Mod);
        assert_mod(1.0, 1.0, 0.0);
        assert_mod(5.0, 3.0, 2.0);
        assert_mod(-1.0, 1.0, 0.0);
        assert_mod(1.0, -1.0, 0.0);
        assert_mod(std::f64::MAX, std::f64::MAX, 0.0);
        assert_mod(std::f64::MIN, std::f64::MIN, 0.0);
    }

    #[test]
    fn op_pow() {
        let assert_pow = assert_arithmetic_op(Opcode::Pow);
        assert_pow(10.0, -1.0, 0.1);
        assert_pow(-1.0, -1.0, -1.0);
        assert_pow(3.0, 2.0, 9.0);
        assert_pow(0.0, 0.0, 1.0);
        assert_pow(std::f64::MAX, std::f64::MAX, std::f64::INFINITY);
        assert_pow(std::f64::MIN, std::f64::MIN, 0.0);
    }

    #[test]
    fn op_or() {
        let assert_or = create_two_operand_assertion(Opcode::Or);

        assert_or(
            Constant::Bool(false),
            Constant::Bool(true),
            RuntimeValue::Bool(true),
        );

        assert_or(
            Constant::Bool(true),
            Constant::Bool(false),
            RuntimeValue::Bool(true),
        );

        assert_or(
            Constant::Bool(false),
            Constant::Bool(false),
            RuntimeValue::Bool(false),
        );

        assert_or(
            Constant::Bool(true),
            Constant::Bool(true),
            RuntimeValue::Bool(true),
        );
    }

    #[test]
    fn op_and() {
        let assert_and = create_two_operand_assertion(Opcode::And);

        assert_and(
            Constant::Bool(false),
            Constant::Bool(true),
            RuntimeValue::Bool(false),
        );

        assert_and(
            Constant::Bool(true),
            Constant::Bool(false),
            RuntimeValue::Bool(false),
        );

        assert_and(
            Constant::Bool(false),
            Constant::Bool(false),
            RuntimeValue::Bool(false),
        );

        assert_and(
            Constant::Bool(true),
            Constant::Bool(true),
            RuntimeValue::Bool(true),
        );
    }

    // End of binary expressions
}
