use bytecode::{
    chunk::{Chunk, ConstantIndex},
    Opcode,
};
use call_frame::CallFrame;
use common::SymbolsReader;
use runtime_error::{RuntimeError, RuntimeErrorCause};
use runtime_value::RuntimeValue;

pub(crate) mod arithmetic;
pub(crate) mod call_frame;
pub(crate) mod runtime_error;
pub(crate) mod runtime_value;
pub(crate) mod stack;

pub type ProgramOutput = Result<RuntimeValue, RuntimeError>;
pub type MachineResult<T> = Result<T, RuntimeError>;
pub type OperationResult = MachineResult<()>;

#[derive(Debug)]
pub(crate) struct VM {
    pub(crate) operands: Vec<RuntimeValue>,
    pub(crate) code: Chunk,
    pub(crate) call_stack: Vec<CallFrame>,
    pub(crate) symbols: SymbolsReader,
    pub(crate) ip: usize,
}

impl VM {
    pub fn new(symbols: SymbolsReader, code: Chunk) -> Self {
        Self {
            operands: Vec::new(),
            call_stack: Vec::new(),
            symbols,
            ip: 0,
            code,
        }
    }

    fn error<T>(&mut self, cause: RuntimeErrorCause) -> MachineResult<T> {
        Err(RuntimeError { cause })
    }

    fn op_constant(&mut self, index: ConstantIndex) -> OperationResult {
        let item = self.code.read(index);
        let value = RuntimeValue::from(item);
        self.operands.push(value);
        Ok(())
    }

    pub fn run(&mut self) -> ProgramOutput {
        loop {
            if self.ip + 1 > self.code.opcodes_len() {
                break;
            }

            let next = self.code.read_opcode(self.ip);

            use Opcode::*;

            let tick = match next {
                Constant(index) => self.op_constant(index),
                Add => self.op_add(),
                Sub => self.op_sub(),
                Mul => self.op_mul(),
                Div => self.op_div(),
                Mod => self.op_mod(),
                Pow => self.op_pow(),

                _ => {
                    todo!();
                }
            }?;

            self.ip += 1;
        }

        self.pop_operand()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bytecode::chunk::Constant;
    use lasso::{Key, Rodeo, Spur};

    fn empty_vm() -> VM {
        new_vm(Chunk::default())
    }

    pub(crate) fn new_vm(code: Chunk) -> VM {
        let symbols = Rodeo::new().into_reader();
        VM::new(symbols, code)
    }

    pub fn assert_program(code: Chunk, expected_outcome: RuntimeValue) {
        let mut vm = new_vm(code);
        assert_eq!(vm.run().unwrap(), expected_outcome);
    }

    #[test]
    fn vm_runs() {
        let mut vm = empty_vm();
        vm.run();
    }

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
}
