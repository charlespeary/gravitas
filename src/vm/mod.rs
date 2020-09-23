use std::collections::HashMap;
use std::ops::Neg;
use std::slice::Iter;

use anyhow::{anyhow, Context, Result};

use crate::bytecode::{Address, Chunk, Number, Opcode, Value};
use crate::settings::Settings;

#[derive(Debug, Default)]
pub struct VM {
    settings: Settings,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
}

type InterpretValue = String;

impl VM {
    fn pop_stack(&mut self) -> Result<Value> {
        self.stack
            .pop()
            .with_context(|| "Tried to pop value from an empty stack")
    }

    fn pop_string(&mut self) -> Result<String> {
        self.pop_stack()?
            .into_string()
            .map_err(|_| anyhow!("Accessed value from the stack that wasn't a string."))
    }

    fn pop_number(&mut self) -> Result<Number> {
        self.pop_stack()?
            .into_number()
            .map_err(|_| anyhow!("Accessed value from the stack that wasn't a number."))
    }

    fn pop_reference(&mut self) -> Result<Address> {
        self.pop_stack()?
            .into_reference()
            .map_err(|_| anyhow!("Accessed value from the stack that wasn't a reference."))
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> Result<InterpretValue> {
        /// Helper to simplify repetitive usage of binary operators
        macro_rules! bin_op {
            // macro for math operations
            ($operator:tt, 'm') => {{
                let a = self.pop_stack()?;
                let b = self.pop_stack()?;
                let result = a $operator b;
                self.stack.push(result?)
            }};
            // macro for logical operations
             ($operator:tt, 'l') => {{
                let a = self.pop_stack()?;
                let b = self.pop_stack()?;
                self.stack.push( Value::Bool(a $operator b))
            }};
        }

        // this line isn't necessary, but somehow the Jetbrains plugin
        // can't infer type for the into_iter on bytecode, so I make the plugin
        // life a little bit easier :)
        let codes: Iter<Opcode> = chunk.into_iter();
        for opcode in codes {
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
                Opcode::Equal => bin_op!(==, 'l'),
                Opcode::Less => bin_op!(<, 'l'),
                Opcode::LessEqual => bin_op!(<=, 'l'),
                Opcode::Greater => bin_op!(>, 'l'),
                Opcode::GreaterEqual => bin_op!(>=, 'l'),
                Opcode::Print => println!("{:?}", self.pop_stack()?),
                Opcode::Assign => {
                    let value = self.pop_stack()?;
                    let address = self.pop_reference()?;
                    self.stack[address as usize] = value;
                }
                Opcode::VarRef(index) => self.stack.push(Value::Reference(*index)),
                Opcode::Var(index) => self.stack.push(self.stack[*index as usize].clone()),
                Opcode::Pop => {
                    self.pop_stack()?;
                }
                Opcode::PopN(amount) => {
                    for _ in 0..*amount {
                        self.pop_stack()?;
                    }
                }
                Opcode::Return => {
                    println!("Return: {:#?}", self.stack.pop());
                }
            }
        }
        Ok(format!("{:#?}", self.stack))
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

// tests are empty for now, because I'm not yet sure whether VM will return the result of the interpretation
#[cfg(test)]
mod test {
    use super::*;

    fn interpret_return() {}

    fn interpret_addition() {}

    fn interpret_subtraction() {}

    fn interpret_multiplication() {}

    fn interpret_division() {}

    fn interpret_negation() {}
}
