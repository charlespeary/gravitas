extern crate derive_more;
#[macro_use]
extern crate lazy_static;

use clap::Clap;

use settings::Settings;
use utils::initialize;

mod chunk;
mod parser;
mod settings;
mod utils;
mod vm;

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
