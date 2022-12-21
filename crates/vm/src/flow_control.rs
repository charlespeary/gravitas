#[cfg(test)]
mod test {
    use bytecode::{
        chunk::{Chunk, Constant},
        Opcode,
    };

    use crate::{
        runtime_error::RuntimeErrorCause, runtime_value::RuntimeValue, test::main_fn,
        OperationResult, VM,
    };

    #[test]
    fn op_jp_forwards() -> OperationResult {
        let code = main_fn(Chunk::new(
            vec![Opcode::Constant(0), Opcode::Jp(10)],
            vec![Constant::Number(127.0)],
        ));

        let mut vm = VM::new();
        assert_eq!(vm.ip, 0);
        // opcodes advance the pointer to 0, and 1 and then we have a jump that advances by another 10 so 11
        assert!(vm.run(code)?.eq(&RuntimeValue::Number(127.0), &mut vm)?);
        assert_eq!(vm.ip, 11);

        Ok(())
    }

    #[test]
    fn op_jp_backwards() -> OperationResult {
        let code = main_fn(Chunk::new(
            vec![Opcode::Constant(0), Opcode::Jp(-3)],
            vec![Constant::Number(127.0)],
        ));

        let mut vm = VM::new();
        assert_eq!(vm.ip, 0);
        // opcodes advance the pointer to 0, 1, and 2 and then we have a jump that retreats by 3 so -1
        // and that will cause a stack overflow
        // If we'd like to just test it to come back to a normal value then it would cause an infinite loop
        // therefore we have to crash the VM in order to check if its doing its job correctly
        assert_eq!(
            vm.run(code).unwrap_err().cause,
            RuntimeErrorCause::StackOverflow
        );

        Ok(())
    }
}
