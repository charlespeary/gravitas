use std::slice::Iter;

use anyhow::{Context, Result};

use crate::bytecode::{Chunk, Opcode, Value};
use crate::settings::Settings;

#[derive(Debug)]
pub struct VM {
    settings: Settings,
    stack: Vec<Value>,
}

/// Helper to simplify repetitive usage of binary operators
// macro_rules! bin_op {
//     ($operator:tt) => {{
//         let a = self.pop_stack()?;
//         let b = self.pop_stack()?;
//         self.stack.push( a $operator b)
//     }}
// }

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

    pub fn interpret(&mut self, chunk: &Chunk) -> Result<String> {
        /// Helper to simplify repetitive usage of binary operators
        macro_rules! bin_op {
            ($operator:tt) => {{
                let a = self.pop_stack()?;
                let b = self.pop_stack()?;
                self.stack.push( a $operator b)
            }}
        }

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
                Opcode::Negate => {
                    let value = self.pop_stack()?;
                    self.stack.push(-value);
                }
                Opcode::Add => bin_op!(+),
                Opcode::Subtract => bin_op!(-),
                Opcode::Divide => bin_op!(/),
                Opcode::Multiply => bin_op!(*),
                Opcode::Return => {
                    println!("Return: {:#?}", self.stack.pop());
                }
            }
        }
        Ok(":D".to_owned())
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
