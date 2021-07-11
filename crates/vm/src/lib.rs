use bytecode::{chunk::Chunk, Opcode};
use call_frame::CallFrame;
use common::SymbolsReader;
use runtime_error::{RuntimeError, RuntimeErrorCause};
use runtime_value::RuntimeValue;

pub(crate) mod basic_expr;
pub(crate) mod call_frame;
pub(crate) mod eq_ord;
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
                Neg => self.op_neg(),
                Not => self.op_not(),
                Eq => self.op_eq(),
                Ne => self.op_ne(),
                Lt => self.op_lt(),
                Le => self.op_le(),
                Gt => self.op_gt(),
                Ge => self.op_ge(),
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
    use lasso::Rodeo;

    fn empty_vm() -> VM {
        new_vm(Chunk::default())
    }

    pub(crate) fn new_vm(code: Chunk) -> VM {
        let symbols = Rodeo::new().into_reader();
        VM::new(symbols, code)
    }

    pub fn assert_program(code: Chunk, expected_outcome: RuntimeValue) {
        let mut vm = new_vm(code);
        assert!(vm.run().unwrap().eq(expected_outcome, &mut vm).unwrap());
    }

    pub(crate) fn create_failable_two_operand_assertion(
        opcode: Opcode,
    ) -> impl Fn(Constant, Constant, RuntimeErrorCause) {
        move |a: Constant, b: Constant, expected: RuntimeErrorCause| {
            let mut vm = new_vm(Chunk::new(
                vec![Opcode::Constant(0), Opcode::Constant(1), opcode],
                vec![a, b],
            ));

            assert_eq!(vm.run().unwrap_err().cause, expected);
        }
    }

    pub(crate) fn create_two_operand_assertion(
        opcode: Opcode,
    ) -> impl Fn(Constant, Constant, RuntimeValue) {
        move |a: Constant, b: Constant, expected: RuntimeValue| {
            let mut vm = new_vm(Chunk::new(
                vec![Opcode::Constant(0), Opcode::Constant(1), opcode],
                vec![a, b],
            ));

            let result = vm.run().unwrap();

            assert!(result.eq(expected, &mut vm).unwrap());
        }
    }

    #[test]
    fn vm_runs() {
        let mut vm = empty_vm();
        vm.run();
    }
}
