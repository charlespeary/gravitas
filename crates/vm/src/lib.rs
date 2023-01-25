use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::Path;

use crate::call::CallType;
use crate::gc::{BoundMethod, HeapObject, Object, Properties};
use bytecode::callables::Function;
use bytecode::stmt::{GlobalItem, GlobalPointer};
use bytecode::{Opcode, ProgramBytecode};
use call::CallFrame;
use common::MAIN_FUNCTION_NAME;
use gc::{Closure, HeapPointer, GC};
use runtime_error::{RuntimeError, RuntimeErrorCause};
use runtime_value::RuntimeValue;

#[macro_use]
extern crate prettytable;

pub(crate) mod basic_expr;
pub(crate) mod call;
pub(crate) mod eq_ord;
pub(crate) mod flow_control;
pub(crate) mod gc;
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

#[derive(Debug)]
struct DebugOptions {
    file: File,
}

impl DebugOptions {
    fn new() -> Self {
        static DEBUG_LOG: &str = "debug.gv";

        if Path::new(DEBUG_LOG).exists() {
            std::fs::remove_file(DEBUG_LOG).unwrap();
        }

        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open("debug.gv")
            .unwrap();

        Self { file }
    }
}

#[derive(Debug)]
pub struct VM {
    pub(crate) operands: Vec<RuntimeValue>,
    pub(crate) call_stack: Vec<CallFrame>,
    pub(crate) ip: usize,
    pub(crate) debug: Option<DebugOptions>,

    pub(crate) globals: Vec<GlobalItem>,
    pub(crate) gc: GC,
}

pub fn run(bytecode: ProgramBytecode, debug: bool) -> RuntimeValue {
    let mut vm = VM::new();

    if debug {
        vm = vm.with_debug();
    }

    vm.run(bytecode).expect("VM went kaboom")
}

impl VM {
    pub fn new() -> Self {
        Self {
            operands: Vec::new(),
            call_stack: vec![],
            ip: 0,
            debug: None,
            globals: vec![],
            gc: GC::new(),
        }
    }

    pub fn with_debug(mut self) -> Self {
        self.debug = Some(DebugOptions::new());
        self
    }

    fn error<T>(&mut self, cause: RuntimeErrorCause) -> MachineResult<T> {
        Err(RuntimeError { cause })
    }

    // TODO: This probably could be hidden behind a feature flag to not
    // decrease VM's performance but since it's not a language to use
    // in real world scenario then it's fine.
    fn debug<S: std::fmt::Display + AsRef<str>>(&mut self, msg: S) {
        if let Some(debug_options) = &mut self.debug {
            if let Err(e) = writeln!(debug_options.file, "{}", msg) {
                eprintln!("Couldn't write to file: {}", e);
            }
        }
    }

    pub(crate) fn current_frame(&self) -> &CallFrame {
        self.call_stack.last().expect("Callstack is empty")
    }

    // TODO: This slows whole execution down, but it's fine for now
    pub(crate) fn current_code(&self) -> &Function {
        let current_frame = self.current_frame();

        let fn_ptr = match self.gc.deref(current_frame.closure_ptr) {
            HeapObject::Closure(closure) => closure.function_ptr,
            HeapObject::BoundMethod(bound_method) => bound_method.method_ptr,
            _ => unreachable!(),
        };

        self.globals.get(fn_ptr).unwrap().as_function()
    }

    pub(crate) fn tick(&mut self) -> MachineResult<TickOutcome> {
        let has_next_opcode = self.ip < self.current_code().chunk.opcodes_len();
        // we finish the program if no next opcode and callstack is empty
        if !has_next_opcode {
            return Ok(TickOutcome::FinishProgram);
        }

        let next = self.current_code().chunk.read_opcode(self.ip);
        println!("next: {:?}", next);
        use Opcode::*;

        self.debug(format!("[OPCODE][NEXT]: {}", &next));

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
                Ok(())
            }
            Pop(amount) => self.op_pop(amount),
            Block(amount) => {
                let block_result = self.pop_operand()?;
                self.op_pop(amount)?;
                self.push_operand(block_result);
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
                let result = self.pop_operand()?;
                self.remove_call_frame();
                self.push_operand(result);
                Ok(())
            }
            Null => {
                self.push_operand(RuntimeValue::Null);
                Ok(())
            }
            CreateClosure(upvalues_count) => {
                let mut upvalues = vec![];

                for _ in 0..upvalues_count {
                    let upvalue_address = self.pop_operand()?.as_address();
                    let upvalue = self.get_variable(upvalue_address.clone())?;
                    let upvalue_ptr = self.gc.allocate(HeapObject::Value(upvalue));
                    upvalues.push(upvalue_ptr);
                }

                let fn_ptr = self.pop_operand()?.as_global_pointer();
                let closure_ptr = self.make_closure(fn_ptr);

                let closure_mut = self.gc.deref_mut(closure_ptr);
                for upvalue in upvalues {
                    match closure_mut {
                        HeapObject::Closure(closure) => {
                            closure.close_upvalue(upvalue);
                        }
                        _ => {
                            todo!();
                        }
                    }
                }
                self.push_operand(RuntimeValue::HeapPointer(closure_ptr));
                Ok(())
            }
            CreateObject(amount) => {
                let mut properties: Properties = HashMap::new();

                for _ in 0..amount {
                    let name = self.pop_operand()?.as_string().clone();
                    let value = self.pop_operand()?;
                    properties.insert(name, value);
                }

                let obj_ptr = self
                    .gc
                    .allocate(HeapObject::Object(Object::new(properties)));

                self.push_operand(RuntimeValue::HeapPointer(obj_ptr));
                Ok(())
            }
            SetProperty(_) => {
                let value = self.pop_operand()?;
                let name = self.pop_operand()?.as_string().clone();
                let obj_ptr = self.pop_operand()?.as_heap_pointer();
                let obj = self.gc.deref_mut(obj_ptr).as_object_mut();
                obj.set(name, value);
                Ok(())
            }
            GetProperty { bind_method } => {
                let name = self.pop_operand()?.as_string().clone();
                let obj_ptr = self.pop_operand()?.as_heap_pointer();
                let obj = self.gc.deref(obj_ptr).as_object();
                self.push_operand(obj.get(&name).clone());

                // let property = if bind_method {
                //     let method_ptr = *self
                //         .deref_global(obj.class_ptr)
                //         .as_class()
                //         .methods
                //         .get(&name)
                //         .expect("Method not found");

                //     let bound_method = BoundMethod {
                //         receiver: obj_ptr,
                //         method_ptr,
                //     };

                //     RuntimeValue::HeapPointer(self.gc.allocate(bound_method.into()))
                // } else {
                //     obj.get(&name).unwrap_or(RuntimeValue::Null)
                // };

                // self.push_operand(property);
                Ok(())
            }
        }?;

        self.move_pointer(1)?;

        Ok(TickOutcome::ContinueExecution)
    }

    pub(crate) fn deref_global(&self, ptr: GlobalPointer) -> &GlobalItem {
        self.globals.get(ptr).unwrap()
    }

    pub(crate) fn make_closure(&mut self, function_ptr: GlobalPointer) -> HeapPointer {
        let closure = Closure {
            function_ptr,
            upvalues: vec![],
        };

        self.gc.allocate(closure.into())
    }

    pub(crate) fn run(&mut self, program: ProgramBytecode) -> ProgramOutput {
        for global in &program.globals {
            self.debug(format!("[GLOBAL][NAME={}]", global.name()));
            self.debug(format!("{}", global));
        }

        self.globals = program.globals;
        let closure_ptr = self.make_closure(program.global_fn_ptr);
        let initial_frame = CallFrame {
            stack_start: 0,
            name: MAIN_FUNCTION_NAME.to_string(),
            closure_ptr,
            return_ip: 0,
        };

        self.add_call_frame(initial_frame);

        self.debug(format!(
            "[VM][START OF EXECUTION][NAME={}]",
            self.current_frame().name
        ));

        loop {
            if self.tick()? == TickOutcome::FinishProgram {
                break;
            }
            self.debug("[VM] TICK");
        }

        self.debug("[VM][END OF EXECUTION]");
        let result = self.pop_operand();
        self.debug(format!("[VM][EXECUTION RESULT][VALUE={:?}]", &result));

        result
    }

    pub(crate) fn move_pointer(&mut self, distance: isize) -> OperationResult {
        use std::ops::Neg;

        self.debug(format!(
            "[VM][MOVE_POINTER][IP_NOW = {}][DISTANCE = {}]",
            self.ip, distance
        ));

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

    pub(crate) fn main_fn(chunk: Chunk) -> Function {
        Function {
            arity: 0,
            chunk,
            name: MAIN_FUNCTION_NAME.to_owned(),
        }
    }

    pub fn assert_program(code: Chunk, expected_outcome: RuntimeValue) {
        let mut vm = VM::new();
        assert!(vm
            .run(main_fn(code))
            .unwrap()
            .eq(&expected_outcome, &mut vm)
            .unwrap());
    }

    pub(crate) fn create_failable_two_operand_assertion(
        opcode: Opcode,
    ) -> impl Fn(Constant, Constant, RuntimeErrorCause) {
        move |a: Constant, b: Constant, expected: RuntimeErrorCause| {
            let mut vm = VM::new();
            let code = main_fn(Chunk::new(
                vec![Opcode::Constant(0), Opcode::Constant(1), opcode],
                vec![a, b],
            ));

            assert_eq!(vm.run(code).unwrap_err().cause, expected);
        }
    }

    pub(crate) fn create_two_operand_assertion(
        opcode: Opcode,
    ) -> impl Fn(Constant, Constant, RuntimeValue) {
        move |a: Constant, b: Constant, expected: RuntimeValue| {
            let mut vm = VM::new();

            let code = main_fn(Chunk::new(
                vec![Opcode::Constant(0), Opcode::Constant(1), opcode],
                vec![a, b],
            ));

            let result = vm.run(code).unwrap();

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
