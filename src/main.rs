mod chunk;
mod debugger;
mod vm;
mod compiler;
mod settings;
mod utils;

use clap::Clap;
use crate::chunk::{Opcode, Chunk};
use crate::debugger::debug_chunk;
use crate::vm::{VM};
use settings::Settings;
use utils::initialize;

fn main() {
    let settings = Settings::parse();
    // let mut vm = VM::from(settings);
    // let mut chunk = Chunk::new();
    // chunk.add_constant(10.0);
    // chunk.add_constant(10.0);
    // chunk.grow(Opcode::Add);
    // chunk.grow(Opcode::Negate);
    // chunk.grow(Opcode::Negate);
    // println!("{:#?}", vm.interpret(&chunk));
    initialize(&settings);
}
