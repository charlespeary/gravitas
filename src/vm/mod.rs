use crate::chunk::{Chunk, Opcode};
use anyhow::Result;
use std::slice::Iter;

pub struct VM;

impl VM {
    pub fn new() -> Self {
        Self
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> Result<String> {

        // this line isn't necessary, but somehow the Jetbrains plugin
        // can't infer type for the into_iter on chunk, so I make the plugin
        // life a little bit easier :)
        let codes: Iter<Opcode> = chunk.into_iter();
        for opcode in codes{
            match opcode {
                Opcode::Constant(index) => {
                    println!("{}", chunk.read_constant(*index));
                }
                Opcode::Return => {
                    println!("Bye!");
                }
            }
        }
        Ok(":D".to_owned())
    }
}
