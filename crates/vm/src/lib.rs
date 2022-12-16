use bytecode::{chunk::Chunk, Opcode, ProgramBytecode};
use call::CallFrame;
use common::MAIN_FUNCTION_NAME;
use runtime_error::{RuntimeError, RuntimeErrorCause};
use runtime_value::RuntimeValue;

pub(crate) mod basic_expr;
pub(crate) mod call;
pub(crate) mod eq_ord;
pub(crate) mod flow_control;
pub(crate) mod memory;
pub(crate) mod runtime_error;
pub(crate) mod runtime_value;
pub(crate) mod stack;

pub type ProgramOutput = Result<RuntimeValue, RuntimeError>;
pub type MachineResult<T> = Result<T, RuntimeError>;
pub type OperationResult = MachineResult<()>;

#[derive(PartialEq)]
pub enum TickOutcome {
    FinishProgram,
    BreakFromLoop,
    ContinueExecution,
}

#[derive(Debug)]
pub struct VM {
    pub(crate) operands: Vec<RuntimeValue>,
    pub(crate) call_stack: Vec<CallFrame>,
    pub(crate) ip: usize,
}

pub fn run(bytecode: ProgramBytecode) -> RuntimeValue {
    let mut vm = VM::new(bytecode);
    vm.run().expect("VM went kaboom")
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        let initial_frame = CallFrame {
            stack_start: 0,
            name: MAIN_FUNCTION_NAME.to_owned(),
            chunk,
        };

        Self {
            operands: Vec::new(),
            call_stack: vec![initial_frame],
            ip: 0,
        }
    }

    fn error<T>(&mut self, cause: RuntimeErrorCause) -> MachineResult<T> {
        Err(RuntimeError { cause })
    }

    pub(crate) fn current_frame(&self) -> &CallFrame {
        self.call_stack.last().expect("Callstack is empty")
    }

    pub(crate) fn end_frame(&mut self) {
        self.call_stack.pop();
    }

    pub(crate) fn tick(&mut self) -> MachineResult<TickOutcome> {
        let has_next_opcode = self.ip < self.current_frame().chunk.opcodes_len();

        if !has_next_opcode && !self.call_stack.is_empty() {
            self.end_frame();
        }

        // we finish the program if no next opcode and callstack is empty
        if !has_next_opcode && self.call_stack.is_empty() {
            return Ok(TickOutcome::FinishProgram);
        }

        let ip = self.ip - self.current_frame().stack_start;

        let next = self.current_frame().chunk.read_opcode(ip);
        use Opcode::*;

        match next {
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
            Or => self.op_or(),
            And => self.op_and(),
            Jif(distance) => {
                let condition = self.pop_operand()?;
                if !condition.to_bool(self)? {
                    self.move_pointer(distance)?;
                }
                return Ok(TickOutcome::BreakFromLoop);
            }
            Jp(distance) => {
                self.move_pointer(distance)?;
                return Ok(TickOutcome::BreakFromLoop);
            }
            Pop(amount) => self.op_pop(amount),
            Get => self.op_get(),
            Asg => self.op_asg(),
            Call => self.op_call(),
            _ => {
                todo!();
            }
        }?;

        self.move_pointer(1)?;

        Ok(TickOutcome::ContinueExecution)
    }

    pub(crate) fn run(&mut self) -> ProgramOutput {
        loop {
            if self.tick()? == TickOutcome::FinishProgram {
                break;
            }
        }
        self.pop_operand()
    }

    pub(crate) fn move_pointer(&mut self, distance: isize) -> OperationResult {
        use std::ops::Neg;

        if distance.is_positive() {
            self.ip += distance as usize;
            Ok(())
        } else {
            match self.ip.checked_sub(distance.neg() as usize) {
                Some(new_ip) => {
                    self.ip = new_ip;
                    Ok(())
                }
                None => self.error(RuntimeErrorCause::StackOverflow),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bytecode::{
        callables::{Class, Function},
        chunk::Constant,
    };
    use common::CONSTRUCTOR_NAME;

    fn empty_vm() -> VM {
        new_vm(Chunk::default())
    }

    pub(crate) fn new_vm(code: Chunk) -> VM {
        VM::new(code)
    }

    pub fn assert_program(code: Chunk, expected_outcome: RuntimeValue) {
        let mut vm = new_vm(code);
        assert!(vm.run().unwrap().eq(&expected_outcome, &mut vm).unwrap());
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

            assert!(result.eq(&expected, &mut vm).unwrap());
        }
    }

    pub(crate) fn dummy_class() -> Class {
        Class {
            name: "dummy".to_owned(),
            constructor: Function {
                arity: 0,
                chunk: Chunk::default(),
                name: CONSTRUCTOR_NAME.to_owned(),
            },
            super_class: None,
            methods: vec![],
        }
    }
}
