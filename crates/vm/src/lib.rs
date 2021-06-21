use bytecode::{
    chunk::{Chunk, ConstantIndex},
    Opcode,
};
use call_frame::CallFrame;
use common::SymbolsReader;
use runtime_error::{RuntimeError, RuntimeErrorCause};
use runtime_value::RuntimeValue;
use stack::Stack;

pub(crate) mod arithmetic;
pub(crate) mod call_frame;
pub(crate) mod runtime_error;
pub(crate) mod runtime_value;
pub(crate) mod stack;

pub type ProgramOutput = Result<RuntimeValue, RuntimeError>;
pub type TickResult<T> = Result<T, RuntimeError>;
pub(crate) struct VM {
    pub(crate) operands: Stack<RuntimeValue>,
    pub(crate) code: Chunk,
    pub(crate) call_stack: Stack<CallFrame>,
    pub(crate) symbols: SymbolsReader,
    pub(crate) ip: usize,
}

impl VM {
    pub fn new(symbols: SymbolsReader, code: Chunk) -> Self {
        Self {
            operands: Stack::new(),
            call_stack: Stack::new(),
            symbols,
            ip: 0,
            code,
        }
    }

    fn error<T>(&mut self, cause: RuntimeErrorCause) -> TickResult<T> {
        Err(RuntimeError { cause })
    }

    fn op_constant(&mut self, index: ConstantIndex) -> TickResult<RuntimeValue> {
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
                _ => {
                    todo!();
                }
            };

            self.ip += 1;
        }

        self.operands.pop()
    }
}

#[cfg(test)]
mod test {
    use bytecode::chunk::Constant;
    use lasso::Rodeo;

    fn empty_vm() -> VM {
        new_vm(Chunk::default())
    }

    fn new_vm(code: Chunk) -> VM {
        let symbols = Rodeo::new().into_reader();
        VM::new(symbols, code)
    }

    use super::*;
    #[test]
    fn vm_runs() {
        let mut vm = empty_vm();
        vm.run();
    }

    #[test]
    fn op_constant() {
        let code = Chunk::new(vec![Opcode::Constant(0)], vec![Constant::Number(10.0)]);
        let mut vm = new_vm(code);

        let res = vm.run().unwrap();
    }
}
