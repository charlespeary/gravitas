use crate::chunk::Chunk;

pub fn debug_chunk(chunk: &Chunk) {
    for opcode in chunk {
        println!("{:#?}", opcode);
    }
}