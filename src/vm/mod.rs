use std::collections::HashMap;
use std::ops::Neg;

use anyhow::{anyhow, Context, Result};

use crate::bytecode::{Address, Chunk, Number, Opcode, Value};
use crate::settings::Settings;
use crate::utils::log;
use crate::vm::call_frame::CallFrame;

mod call_frame;

#[derive(Debug, Default)]
pub struct VM<'a> {
    settings: Settings,
    stack: Vec<Value>,
    call_stack: Vec<CallFrame<'a>>,
    globals: HashMap<String, Value>,
    ip: usize,
}

type InterpretValue = String;

impl<'a> VM<'a> {
    fn pop_stack(&mut self) -> Result<Value> {
        let value = self.stack.pop();
        if self.settings.debug {
            log::vm_subtitle("POP STACK", &value);
        }
        value.with_context(|| "Tried to pop value from an empty stack")
    }

    fn pop_reference(&mut self) -> Result<Address> {
        self.pop_stack()?
            .into_reference()
            .map_err(|_| anyhow!("Accessed value from the stack that wasn't a reference."))
    }

    fn drop(&mut self, amount: usize) -> Result<()> {
        for _ in 0..amount {
            self.pop_stack()?;
        }
        Ok(())
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> Result<InterpretValue> {
        log::title_success("INTERPRETATION");
        /// Helper to simplify repetitive usage of binary operators
        macro_rules! bin_op {
            // macro for math operations
            ($operator:tt, 'm') => {{
                let a = self.pop_stack()?;
                let b = self.pop_stack()?;
                let result = b $operator a;
                self.stack.push(result?)
            }};
            // macro for logical operations
             ($operator:tt, 'l') => {{
                let a = self.pop_stack()?;
                let b = self.pop_stack()?;

                self.stack.push( Value::Bool(b $operator a))
            }};
        }

        while let Some(opcode) = chunk.code.get(self.ip) {
            if self.settings.debug {
                log::vm_title("OPCODE", opcode);
                log::vm_subtitle("IP", &self.ip);
                log::vm_subtitle("STACK", &self.stack);
            }
            match opcode {
                Opcode::Constant(index) => self.stack.push(chunk.read_constant(*index).clone()),
                Opcode::True => self.stack.push(Value::Bool(true)),
                Opcode::False => self.stack.push(Value::Bool(false)),
                Opcode::Null => self.stack.push(Value::Null),
                Opcode::Negate => {
                    let value = self.pop_stack()?;
                    let negated = value.neg()?;
                    self.stack.push(negated);
                }
                Opcode::Not => {
                    let value: bool = self.pop_stack()?.into();
                    self.stack.push(Value::Bool(value));
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
                Opcode::Print => println!("{:#?}", self.pop_stack()?),
                Opcode::Assign => {
                    let value = self.pop_stack()?;
                    let address = self.pop_reference()?;
                    self.stack[address as usize] = value;
                    self.stack.push(Value::Null);
                }
                Opcode::VarRef(index) => self.stack.push(Value::Reference(*index)),
                Opcode::Var(index) => self.stack.push(self.stack[*index as usize].clone()),
                Opcode::PopN(amount) => {
                    self.drop(*amount)?;
                }
                Opcode::JumpIfFalse(jump) => {
                    let value: bool = self.pop_stack()?.into();
                    if !value {
                        self.ip += (*jump) as usize;
                    }
                }
                Opcode::JumpForward(jump) => {
                    self.ip += (*jump) as usize;
                }
                Opcode::JumpBack(jump) => {
                    self.ip -= *jump as usize;
                    continue;
                }
                Opcode::Block(declared) => {
                    let result = self.pop_stack()?;
                    self.drop(*declared)?;
                    self.stack.push(result);
                }
                Opcode::Break(jump) => {
                    let value = self.pop_stack()?;
                    self.ip += *jump as usize;
                    self.stack.push(value);
                }
                Opcode::Return => {
                    println!("Return: {:#?}", self.stack.pop());
                }
            }
            self.ip += 1;
        }
        Ok(format!("{:#?}", self.stack))
    }
}

impl<'a> From<Settings> for VM<'a> {
    fn from(settings: Settings) -> Self {
        VM {
            settings,
            ..Default::default()
        }
    }
}
