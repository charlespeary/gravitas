use crate::chunk::{Opcode, Chunk};
use crate::debugger::debug_chunk;
use crate::vm::VM;

mod chunk;
mod debugger;
mod vm;

fn main() {
    let mut vm = VM::new();
    let mut chunk = Chunk::new();
    chunk.add_constant(10.0);
    chunk.grow(Opcode::Return);
    vm.interpret(&chunk);
    debug_chunk(&chunk);
}
