use crate::{runtime_value::RuntimeValue, OperationResult, VM};

impl VM {
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
}

#[cfg(test)]
mod test {
    use bytecode::{
        chunk::{Chunk, Constant},
        Opcode,
    };

    use crate::{runtime_error::RuntimeErrorCause, runtime_value::RuntimeValue, test::new_vm};

    fn assert_arithmetic_op(opcode: Opcode) -> impl Fn(f64, f64, f64) {
        move |a: f64, b: f64, expected: f64| {
            let mut vm = new_vm(Chunk::new(
                vec![Opcode::Constant(0), Opcode::Constant(1), opcode],
                vec![Constant::Number(a), Constant::Number(b)],
            ));

            assert_eq!(vm.run().unwrap(), RuntimeValue::Number(expected));
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
}
