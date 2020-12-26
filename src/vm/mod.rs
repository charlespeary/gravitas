use std::collections::HashMap;
use std::ops::Neg;

use anyhow::{anyhow, Context, Result};

use crate::{
    bytecode::{Address, Callable, Chunk, Opcode, Value},
    cli::{commands::test::TestRunner, Settings},
    compiler::ProgramOutput,
    std::GLOBALS,
    utils::logger,
    vm::{
        call_frame::{CallFrame, Environments},
        utilities::Utilities,
    },
};

mod call_frame;
pub mod utilities;

#[derive(Debug, Default)]
pub struct VM<'a> {
    pub utilities: Option<&'a mut Utilities<'a>>,
    // Stack of local values
    stack: Vec<Value>,
    // Struct managing environments
    environments: Environments,
    // Stack of function calls
    call_stack: Vec<CallFrame>,
    // Hashmap of global variables, e.g global functions
    globals: HashMap<String, Value>,
    ip: usize,
}

impl<'a> VM<'a> {
    fn push_stack(&mut self, value: Value) {
        self.stack.push(value)
    }

    fn truncate_stack(&mut self, to: usize) {
        self.stack.truncate(to);
    }

    fn pop_stack(&mut self) -> Result<Value> {
        let value = self.stack.pop();
        value.with_context(|| "Tried to pop value from an empty stack")
    }

    fn pop_address(&mut self) -> Result<Address> {
        self.pop_stack()?
            .into_address()
            .map_err(|_| anyhow!("Accessed value from the stack that wasn't a reference."))
    }

    fn pop_callable(&mut self) -> Result<Callable> {
        self.pop_stack()?
            .into_callable()
            .map_err(|_| anyhow!("Accessed value from the stack that wasn't callable."))
    }

    fn pop_n(&mut self, n: usize) -> Result<Vec<Value>> {
        let mut values: Vec<Value> = vec![];

        for _ in 0..n {
            values.push(self.pop_stack()?);
        }
        Ok(values)
    }

    pub fn new_frame(&mut self, chunk: Chunk, arity: usize, caller_name: String, env_key: usize) {
        self.call_stack.push(CallFrame {
            chunk,
            // Function should have access to its arguments (arity)
            // in order to allow recursive calls
            stack_start: self.stack.len() - arity,
            // +1 so we skip the opcode that caused call
            return_address: self.ip + 1,
            env_key,
            caller_name,
        });
    }

    fn lookup_call_frame(&self, depth: usize) -> &CallFrame {
        self.call_stack
            .get(self.call_stack.len() - 1 - depth)
            .expect("Tried to lookup call frame above the call stack")
    }

    pub fn callback(&mut self, callable: Callable) -> Result<()> {
        match &callable {
            Callable::NativeFunction(_) => {
                self.call(callable);
            }
            _ => {
                self.call(callable);
                self.evaluate_last_frame();
            }
        }
        Ok(())
    }

    pub fn call(&mut self, callable: Callable) -> Result<()> {
        let current_env = self.lookup_call_frame(0).env_key;
        match callable {
            Callable::Function(function) => {
                let env_key = self.environments.create_env(current_env);
                self.new_frame(
                    function.chunk.clone(),
                    function.arity,
                    function.name.clone(),
                    env_key,
                );
                self.stack.push(Callable::Function(function).into());
                self.ip = 0;
                Ok(())
            }
            Callable::NativeFunction(function) => {
                let args = self.pop_n(function.arity)?;
                let value = (function.function)(args, self);
                self.stack.push(value);
                // skip the call
                self.ip += 1;
                Ok(())
            }
            Callable::Closure(closure) => {
                // Create new environment to hold closed values in
                let env = self.environments.create_env(
                    closure
                        .enclosing_env_key
                        .expect("Closure must be enclosed at least by the global env"),
                );
                self.new_frame(
                    closure.chunk.clone(),
                    closure.arity,
                    String::from("lambda"),
                    env,
                );

                self.stack
                    .push(Callable::Closure(closure.with_env(env)).into());
                self.ip = 0;
                Ok(())
            }
        }
    }

    pub fn close_frame(&mut self) {
        let frame = self
            .call_stack
            .pop()
            .expect("Tried to end call, but there were no call frames available.");
        self.ip = frame.return_address;
        self.truncate_stack(frame.stack_start);
    }

    pub fn interpret(&mut self, chunk: Chunk) -> Result<Value> {
        // Reset structs values in case if we would like to rerun some code on VM
        self.ip = 0;
        self.environments = Environments::default();
        self.globals = HashMap::default();
        self.call_stack = vec![];

        self.call_stack.push(CallFrame {
            chunk,
            stack_start: self.stack.len(),
            return_address: self.ip,
            caller_name: String::from("main"),
            env_key: 0,
        });
        self.run()
    }

    pub fn evaluate_last_frame(&mut self) -> Result<()> {
        let frame = self.call_stack.last().cloned().expect("Tried to pop from empty callstack.");
        self.evaluate_frame(frame)
    }

    pub fn evaluate_frame(&mut self, frame: CallFrame) -> Result<()> {
        /// Helper to simplify repetitive usage of binary operators
        macro_rules! bin_op {
            // macro for math operations
            ($operator:tt, 'm') => {{
                let a = self.pop_stack()?;
                let b = self.pop_stack()?;
                let result = b $operator a;
                self.push_stack(result?)
            }};
            // macro for logical operations
             ($operator:tt, 'l') => {{
                let a = self.pop_stack()?;
                let b = self.pop_stack()?;

                self.push_stack( Value::Bool(b $operator a))
            }};
        }

        while let Some(opcode) = frame.chunk.code.get(self.ip) {
            match opcode {
                Opcode::Constant(index) => {
                    self.push_stack(frame.chunk.read_constant(*index).clone())
                }
                Opcode::True => self.push_stack(Value::Bool(true)),
                Opcode::False => self.push_stack(Value::Bool(false)),
                Opcode::Null => self.push_stack(Value::Null),
                Opcode::Negate => {
                    let value = self.pop_stack()?;
                    let negated = value.neg()?;
                    self.push_stack(negated);
                }
                Opcode::Not => {
                    let value: bool = self.pop_stack()?.into();
                    self.push_stack(Value::Bool(value));
                }
                Opcode::Add => bin_op!(+, 'm'),
                Opcode::Subtract => bin_op!(-, 'm'),
                Opcode::Divide => bin_op!(/, 'm'),
                Opcode::Multiply => bin_op!(*, 'm'),
                Opcode::BangEqual => bin_op!(!=, 'l'),
                Opcode::Compare => bin_op!(==, 'l'),
                Opcode::Less => bin_op!(<, 'l'),
                Opcode::LessEqual => bin_op!(<=, 'l'),
                Opcode::Greater => bin_op!(>, 'l'),
                Opcode::GreaterEqual => bin_op!(>=, 'l'),
                Opcode::Assign => {
                    let value = self.pop_stack()?;

                    match self.pop_address()? {
                        Address::Local(address) => {
                            self.stack[frame.stack_start + address] = value;
                        }
                        Address::Upvalue(index, depth) => {
                            match self.environments.get_value_mut(frame.env_key, index, depth) {
                                Some(old_value) => {
                                    let _ = std::mem::replace(old_value, value);
                                }
                                // If it wasn't yet closed then it must live on the stack
                                None => {
                                    let address =
                                        self.lookup_call_frame(depth).stack_start + index;
                                    self.stack[address] = value;
                                }
                            }
                        }
                        Address::Global(_) => {
                            return Err(anyhow!("Cannot reassign global resource!"));
                        }
                    };
                    self.push_stack(Value::Null);
                }
                Opcode::PopN(amount) => {
                    self.pop_n(*amount)?;
                }
                Opcode::JumpIfFalse(jump) => {
                    let value: bool = self.pop_stack()?.into();
                    if !value {
                        self.ip += *jump;
                    }
                }
                Opcode::JumpForward(jump) => {
                    self.ip += *jump;
                }
                Opcode::JumpBack(jump) => {
                    self.ip -= *jump;
                    continue;
                }
                Opcode::Block(declared) => {
                    let result = self.pop_stack()?;
                    self.pop_n(*declared)?;
                    self.push_stack(result);
                }
                Opcode::CloseValue => {
                    let address = self
                        .pop_address()?
                        .into_local()
                        .map_err(|_| anyhow!("ds"))?;
                    let value = self.stack[frame.stack_start + address].clone();
                    self.environments.close_value(value, frame.env_key);
                }
                Opcode::CreateClosure => {
                    let current_env = self.lookup_call_frame(0).env_key;
                    let closure = self
                        .pop_callable()?
                        .into_closure()
                        .map_err(|_| anyhow!("Popped callable that wasn't a closure."))?;

                    self.push_stack(closure.with_enclosing_env(current_env).into());
                }
                Opcode::Break(jump) => {
                    let value = self.pop_stack()?;
                    self.ip += *jump;
                    self.push_stack(value);
                }
                Opcode::Return => {
                    let result_value = self.pop_stack()?;
                    self.close_frame();
                    self.push_stack(result_value);
                    return Ok(());
                }
                Opcode::Get => {
                    let address = self.pop_address()?;
                    let value = match address {
                        Address::Local(index) => self.stack[frame.stack_start + index].clone(),
                        Address::Upvalue(index, depth) => {
                            match self.environments.get_value(frame.env_key, index, depth) {
                                Some(value) => value,
                                // If it wasn't yet closed then it must live on the stack
                                None => {
                                    let address =
                                        self.lookup_call_frame(depth).stack_start + index;
                                    self.stack[address].clone()
                                }
                            }
                        }
                        Address::Global(name) => GLOBALS
                            .get(name.as_str())
                            .cloned()
                            .with_context(|| anyhow!("Global variable {} doesn't exist", name))?
                            .into(),
                    };
                    self.push_stack(value);
                }
                Opcode::Call => {
                    let caller = self.pop_callable()?;
                    self.call(caller)?;
                    return Ok(());
                }
            }
            self.ip += 1;
        }

        self.close_frame();

        Ok(())
    }

    pub fn run(&mut self) -> ProgramOutput {
        'frames: while let Some(frame) = self.call_stack.last_mut().cloned() {
            self.evaluate_frame(frame);
        }

        Ok(Value::Null)
    }

    pub fn with_utilities(mut self, utilities: &'a mut Utilities<'a>) -> Self {
        self.utilities = Some(utilities);
        self
    }
}


#[cfg(test)]
mod test {}
