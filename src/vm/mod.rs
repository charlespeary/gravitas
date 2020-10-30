use std::collections::HashMap;
use std::ops::Neg;

use anyhow::{anyhow, Context, Result};

use crate::{
    bytecode::{stmt::function::Function, Address, Callable, Chunk, Number, Opcode, Value},
    settings::Settings,
    std::GLOBALS,
    utils::log,
    vm::call_frame::CallFrame,
};

mod call_frame;

#[derive(Debug, Default)]
pub struct VM {
    settings: Settings,
    stack: Vec<Value>,
    call_stack: Vec<CallFrame>,
    globals: HashMap<String, Value>,
    ip: usize,
}

impl VM {
    fn push_stack(&mut self, value: Value) {
        if self.settings.debug {
            log::vm_subtitle("PUSH STACK", &value);
        }
        self.stack.push(value)
    }

    fn truncate_stack(&mut self, to: usize) {
        if self.settings.debug {
            log::vm_subtitle("TRUNCATE STACK", &to);
        }
        self.stack.truncate(to);
    }

    fn pop_stack(&mut self) -> Result<Value> {
        let value = self.stack.pop();
        if self.settings.debug {
            log::vm_subtitle("POP STACK", &value);
        }
        value.with_context(|| "Tried to pop value from an empty stack")
    }

    fn pop_string(&mut self) -> Result<String> {
        self.pop_stack()?
            .into_string()
            .map_err(|_| anyhow!("Accessed value from the stack that wasn't a string."))
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

    pub fn call(&mut self, callable: Callable) -> Result<()> {
        match callable.clone() {
            Callable::Function(function) => {
                let Function { chunk, arity, .. } = function;
                if self.settings.debug {
                    log::vm_subtitle("CALL BODY", &chunk);
                }
                self.call_stack.push(CallFrame {
                    chunk,
                    // Function should have access to its arguments (arity)
                    // in order to allow recursive calls
                    stack_start: self.stack.len() - arity,
                    // +1 so we skip the opcode that caused call
                    return_address: self.ip + 1,
                });
                self.stack.push(callable.into());
                self.ip = 0;
                Ok(())
            }
            Callable::NativeFunction(function) => {
                let args = self.pop_n(function.arity)?;
                let value = (function.function)(args);
                self.stack.push(value);
                // skip the call
                self.ip += 1;
                Ok(())
            }
        }
    }

    pub fn end_call(&mut self) {
        let frame = self
            .call_stack
            .pop()
            .expect("Tried to end call, but there were no call frames available.");
        self.ip = frame.return_address;
        self.truncate_stack(frame.stack_start);
    }

    pub fn interpret(&mut self, chunk: Chunk) -> Result<Value> {
        self.call_stack.push(CallFrame {
            chunk,
            stack_start: self.stack.len(),
            return_address: self.ip,
        });
        self.run()
    }

    pub fn run(&mut self) -> Result<Value> {
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

        // We can safely unwrap, because there will always be at least one call frame on the stack
        'frames: while let Some(frame) = self.call_stack.last().cloned() {
            while let Some(opcode) = frame.chunk.code.get(self.ip) {
                if self.settings.debug {
                    log::vm_title("OPCODE", opcode);
                    log::vm_subtitle("IP", &self.ip);
                    log::vm_subtitle("STACK", &self.stack);
                }

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
                        let address = self
                            .pop_address()?
                            .into_local()
                            .map_err(|_| anyhow!("Cannot reassign global resource!"))?;
                        self.stack[frame.stack_start + address] = value;
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
                    Opcode::Break(jump) => {
                        let value = self.pop_stack()?;
                        self.ip += *jump;
                        self.push_stack(value);
                    }
                    Opcode::Return => {
                        let result_value = self.pop_stack()?;
                        self.end_call();
                        self.push_stack(result_value);
                        continue 'frames;
                    }
                    Opcode::Get => {
                        let address = self.pop_address()?;
                        let value = match address {
                            Address::Local(index) => self.stack[frame.stack_start + index].clone(),
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
                        continue 'frames;
                    }
                }
                self.ip += 1;
            }
            self.end_call();
        }
        // Reset ip so it points to the beginning of new chunk
        Ok(Value::Null)
    }
}

impl From<Settings> for VM {
    fn from(settings: Settings) -> Self {
        VM {
            settings,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod test {}
