use std::ops::Neg;

use bytecode::chunk::ConstantIndex;

use crate::{runtime_value::RuntimeValue, OperationResult, VM};

impl VM {
    // Start of stuff that doesn't belong to any particular group

    pub(crate) fn op_constant(&mut self, index: ConstantIndex) -> OperationResult {
        let item = self.code.read(index);
        let value = RuntimeValue::from(item);
        self.operands.push(value);
        Ok(())
    }

    // End of stuff that doesn't belong to any particular group

    // Start of unary expressions

    pub(crate) fn op_neg(&mut self) -> OperationResult {
        let a = self.pop_number()?;
        self.operands.push(RuntimeValue::Number(a.neg()));
        Ok(())
    }

    pub(crate) fn op_not(&mut self) -> OperationResult {
        let a = self.pop_bool()?;
        self.operands.push(RuntimeValue::Bool(!a));
        Ok(())
    }

    // End of unary expressions

    // Start of binary expressions
    pub(crate) fn op_add(&mut self) -> OperationResult {
        let a = self.pop_number()?;
        let b = self.pop_number()?;
        self.operands.push(RuntimeValue::Number(a + b));
        Ok(())
    }

    pub(crate) fn op_sub(&mut self) -> OperationResult {
        let a = self.pop_number()?;
        let b = self.pop_number()?;
        self.operands.push(RuntimeValue::Number(a - b));
        Ok(())
    }

    pub(crate) fn op_mul(&mut self) -> OperationResult {
        let a = self.pop_number()?;
        let b = self.pop_number()?;
        self.operands.push(RuntimeValue::Number(a * b));
        Ok(())
    }

    pub(crate) fn op_div(&mut self) -> OperationResult {
        let a = self.pop_number()?;
        let b = self.pop_number()?;
        self.operands.push(RuntimeValue::Number(a / b));
        Ok(())
    }

    pub(crate) fn op_mod(&mut self) -> OperationResult {
        let a = self.pop_number()?;
        let b = self.pop_number()?;
        self.operands.push(RuntimeValue::Number(a % b));
        Ok(())
    }

    pub(crate) fn op_pow(&mut self) -> OperationResult {
        let a = self.pop_number()?;
        let b = self.pop_number()?;
        self.operands.push(RuntimeValue::Number(a.powf(b)));
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

    use lasso::{Key, Spur};

    use crate::{
        runtime_error::RuntimeErrorCause,
        runtime_value::RuntimeValue,
        test::{assert_program, create_two_operand_assertion, new_vm},
    };

    // Start of stuff that doesn't belong to any particular group

    #[test]
    fn op_constant() {
        fn assert_constant(constant: Constant) {
            assert_program(
                Chunk::new(vec![Opcode::Constant(0)], vec![constant]),
                RuntimeValue::from(constant),
            );
        }

        assert_constant(Constant::Bool(false));
        assert_constant(Constant::Bool(true));
        assert_constant(Constant::String(Spur::try_from_usize(0).unwrap()));
        assert_constant(Constant::Number(std::f64::MAX));
        assert_constant(Constant::Number(std::f64::MIN));
    }

    // End of stuff that doesn't belong to any particular group

    // Start of unary expressions

    #[test]
    fn op_neg() {
        // Accept only booleans
        let mut vm = new_vm(Chunk::new(
            vec![Opcode::Constant(0), Opcode::Neg],
            vec![Constant::Bool(true)],
        ));

        assert_eq!(
            vm.run().unwrap_err().cause,
            RuntimeErrorCause::ExpectedNumber
        );

        let assert_neg = |a, e| {
            let mut vm = new_vm(Chunk::new(
                vec![Opcode::Constant(0), Opcode::Neg],
                vec![Constant::Number(a)],
            ));
            assert_eq!(vm.run().unwrap(), RuntimeValue::Number(e));
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

        assert_eq!(vm.run().unwrap_err().cause, RuntimeErrorCause::ExpectedBool);

        let assert_not = |a, e| {
            let mut vm = new_vm(Chunk::new(
                vec![Opcode::Constant(0), Opcode::Not],
                vec![Constant::Bool(a)],
            ));
            assert_eq!(vm.run().unwrap(), RuntimeValue::Bool(e));
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
                RuntimeErrorCause::ExpectedNumber
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
        assert_sub(0.0, 10.0, 10.0);
        assert_sub(10.0, 0.0, -10.0);
        assert_sub(std::f64::MIN, std::f64::MIN, 0.0);
        assert_sub(std::f64::MAX, std::f64::MAX, 0.0);
        dbg!(std::f64::MAX, std::f64::MIN);
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
        assert_div(1.0, 10.0, 10.0);
        assert_div(-1.0, -1.0, 1.0);
    }

    #[test]
    fn op_mod() {
        let assert_mod = assert_arithmetic_op(Opcode::Mod);
        assert_mod(1.0, 1.0, 0.0);
        assert_mod(3.0, 5.0, 2.0);
        assert_mod(1.0, -1.0, 0.0);
        assert_mod(-1.0, 1.0, 0.0);
        assert_mod(std::f64::MAX, std::f64::MAX, 0.0);
        assert_mod(std::f64::MIN, std::f64::MIN, 0.0);
    }

    #[test]
    fn op_pow() {
        let assert_pow = assert_arithmetic_op(Opcode::Pow);
        assert_pow(-1.0, 10.0, 0.1);
        assert_pow(-1.0, -1.0, -1.0);
        assert_pow(2.0, 3.0, 9.0);
        assert_pow(0.0, 0.0, 1.0);
        assert_pow(std::f64::MAX, std::f64::MAX, std::f64::INFINITY);
        assert_pow(std::f64::MIN, std::f64::MIN, 0.0);
    }

    // End of binary expressions
}
