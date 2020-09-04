mod settings;

pub use settings::VMSettings;
use crate::chunk::{Chunk, Opcode};
use anyhow::Result;
use std::slice::Iter;

#[derive(Debug)]
pub struct VM {
    settings: VMSettings
}

impl VM {
    pub fn new() -> Self {
        Self {
            settings: VMSettings::default()
        }
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> Result<String> {

        // this line isn't necessary, but somehow the Jetbrains plugin
        // can't infer type for the into_iter on chunk, so I make the plugin
        // life a little bit easier :)
        let codes: Iter<Opcode> = chunk.into_iter();
        for opcode in codes {
            if self.settings.debug {
                println!("=== {:?} ===", opcode);
            }
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


impl From<VMSettings> for VM {
    fn from(settings: VMSettings) -> Self {
        VM {
            settings
        }
    }
}