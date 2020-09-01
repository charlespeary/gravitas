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
    writeChunk(&chunk, RETURN, 0);
    writeConstant(&chunk, 1.2, 1);
//    writeConstant(&chunk, 4.2, 2);
//    writeConstant(&chunk, 3.2, 3);
    disassembleChunk(&chunk, "test chunk");
    freeChunk(&chunk);
    return 0;
}
