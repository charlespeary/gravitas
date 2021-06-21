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
}

#[cfg(test)]
mod test {
    use bytecode::{
        chunk::{Chunk, Constant},
        Opcode,
    };

    use crate::{runtime_error::RuntimeErrorCause, runtime_value::RuntimeValue, test::new_vm};

    fn assert_arithmetic_op(a: f64, b: f64, expected: f64, opcode: Opcode) {
        let mut vm = new_vm(Chunk::new(
            vec![Opcode::Constant(0), Opcode::Constant(1), opcode],
            vec![Constant::Number(a), Constant::Number(b)],
        ));

        assert_eq!(vm.run().unwrap(), RuntimeValue::Number(expected));
    }

    #[test]
    fn op_add() {
        let assert_add = |a, b, e| assert_arithmetic_op(a, b, e, Opcode::Add);

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
    }

    #[test]
    fn op_sub() {
        let assert_sub = |a, b, e| assert_arithmetic_op(a, b, e, Opcode::Sub);

        // Expect 10.0 to be on top of the stack
        assert_sub(0.0, 10.0, 10.0);
        assert_sub(10.0, 0.0, -10.0);
        assert_sub(std::f64::MIN, std::f64::MIN, 0.0);
        assert_sub(std::f64::MAX, std::f64::MAX, 0.0);
        dbg!(std::f64::MAX, std::f64::MIN);
        assert_sub(std::f64::MIN, -std::f64::MAX, 0.0);
    }
}
