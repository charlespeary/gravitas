use std::ops::Neg;
use std::slice::Iter;

use anyhow::{Context, Result};

use crate::bytecode::{Chunk, Opcode, Value};
use crate::settings::Settings;

#[derive(Debug)]
pub struct VM {
    settings: Settings,
    stack: Vec<Value>,
}

type InterpretValue = String;

impl VM {
    pub fn new() -> Self {
        Self {
            settings: Settings::default(),
            stack: Vec::new(),
        }
    }

    fn pop_stack(&mut self) -> Result<Value> {
        self.stack
            .pop()
            .with_context(|| "Tried to pop value from an empty stack")
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> Result<InterpretValue> {
        /// Helper to simplify repetitive usage of binary operators
        macro_rules! bin_op {
            // macro for math operations
            ($operator:tt, 'm') => {{
                let a = self.pop_stack()?.as_number()?;
                let b = self.pop_stack()?.as_number()?;
                self.stack.push( Value::Number(a $operator b))
            }};
            // macro for logical operations
             ($operator:tt, 'l') => {{
                let a = self.pop_stack()?;
                let b = self.pop_stack()?;
                self.stack.push( Value::Bool(a $operator b))
            }};
        }

        // macro_rules! compare {
        //     ($operator:tt) => {{
        //         let a = self.pop_stack()?.as_number()?;
        //         let b = self.pop_stack()?.as_number()?;
        //         self.stack.push( Value::Bool(a $operator b))
        //     }}
        // }

        // this line isn't necessary, but somehow the Jetbrains plugin
        // can't infer type for the into_iter on bytecode, so I make the plugin
        // life a little bit easier :)
        let codes: Iter<Opcode> = chunk.into_iter();
        for opcode in codes {
            if self.settings.debug {
                println!("=== {:?} ===", opcode);
                println!("=== STACK ===");
                println!("{:?}", self.stack);
            }

            match opcode {
                Opcode::Constant(index) => {
                    self.stack.push(chunk.read_constant(*index));
                }
                Opcode::True => {
                    println!("That's true");
                }
                Opcode::False => {
                    println!("That's false");
                }
                Opcode::Null => {
                    println!("boo! null...");
                }
                Opcode::Negate => {
                    let value = self.pop_stack()?;
                    let negated = value.neg()?;
                    self.stack.push(negated);
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
            stack: Vec::new(),
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
