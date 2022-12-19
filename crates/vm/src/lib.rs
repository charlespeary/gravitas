use crate::call::CallType;
use bytecode::{
    chunk::{Chunk, Constant},
    Opcode, ProgramBytecode,
};
use call::CallFrame;
use common::MAIN_FUNCTION_NAME;
use runtime_error::{RuntimeError, RuntimeErrorCause};
use runtime_value::RuntimeValue;
#[macro_use]
extern crate prettytable;
use prettytable::{Cell, Row, Table};

pub(crate) mod basic_expr;
pub(crate) mod call;
pub(crate) mod eq_ord;
pub(crate) mod flow_control;
pub mod gravitas_std;
pub(crate) mod memory;
pub(crate) mod runtime_error;
pub mod runtime_value;
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

#[derive(Debug, Clone)]
pub struct VM {
    pub(crate) operands: Vec<RuntimeValue>,
    pub(crate) call_stack: Vec<CallFrame>,
    pub(crate) ip: usize,
}

pub fn run(bytecode: ProgramBytecode, debug: bool) -> RuntimeValue {
    if debug {
        let mut table = Table::new();

        table.add_row(row!["OPCODE", "CONSTANT INDEX", "CONSTANT VALUE"]);

        let opcodes = bytecode.opcodes.iter();
        let constants = bytecode
            .constants
            .iter()
            .enumerate()
            .map(|(i, c)| (i.to_string(), c.to_string()))
            .chain(std::iter::repeat(("-".to_owned(), "-".to_owned())));

        for (opcode, (constant_index, constant_value)) in opcodes.zip(constants) {
            table.add_row(row![
                opcode.to_string(),
                constant_index,
                constant_value.to_string()
            ]);
        }

        // Print the table to stdout
        table.printstd();
    }

    let mut vm = VM::new(bytecode);
    vm.run().expect("VM went kaboom")
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        let initial_frame = CallFrame {
            stack_start: 0,
            name: MAIN_FUNCTION_NAME.to_owned(),
            chunk,
            return_ip: 0,
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

    pub(crate) fn tick(&mut self) -> MachineResult<TickOutcome> {
        let has_next_opcode = self.ip < self.current_frame().chunk.opcodes_len();

        // we finish the program if no next opcode and callstack is empty
        if !has_next_opcode {
            return Ok(TickOutcome::FinishProgram);
        }

        let next = self.current_frame().chunk.read_opcode(self.ip);
        use Opcode::*;
        println!("N: {}", next);

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
                Ok(())
            }
            Jp(distance) => {
                self.move_pointer(distance)?;
                // So we don't increment the IP after jumping
                return Ok(TickOutcome::ContinueExecution);
            }
            Pop(amount) => self.op_pop(amount),
            Block(amount) => {
                let block_result = self.pop_operand()?;
                self.op_pop(amount)?;
                self.operands.push(block_result);
                Ok(())
            }
            Break(distance) => {
                self.move_pointer(distance)?;
                Ok(())
            }
            Get => self.op_get(),
            Asg => self.op_asg(),
            Call => match self.op_call()? {
                CallType::EnterFnBody => {
                    self.ip = 0;
                    return Ok(TickOutcome::ContinueExecution);
                }
                CallType::InlineFn => Ok(()),
            },
            Return => {
                println!("before stack: {:?}", &self.operands);
                let result = self.pop_operand()?;
                self.remove_call_frame();
                println!("after stack: {:?}", &self.operands);
                self.operands.push(result);
                Ok(())
            }
            Null => {
                self.operands.push(RuntimeValue::Null);
                Ok(())
            }
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
