#include <stdio.h>

#ifndef clox_common_h
#define clox_common_h

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include "chunk/chunk.h"
#include "disassembler/disassembler.h"

#endif

int main() {
    Chunk chunk;
    initChunk(&chunk);
    writeChunk(&chunk, RETURN);
    int constant = addConstant(&chunk, 1.2);
    writeChunk(&chunk, CONSTANT);
    writeChunk(&chunk, constant);
    disassembleChunk(&chunk, "test chunk");
    freeChunk(&chunk);
    return 0;
}
