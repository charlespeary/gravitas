use crate::chunk::Chunk;

pub fn debug_chunk(chunk: &Chunk, name: &str) {
    println!("=== {} ===", name);
    for opcode in chunk {
        println!("{:#?}", opcode);
    }
}