//
// Created by karolgruszka on 27.08.2020.
//
#include <stdlib.h>
#include <stdio.h>
#include "chunk.h"
#include "../memory/memory.h"

void initChunk(Chunk *chunk) {
    chunk->capacity = 0;
    chunk->count = 0;
    chunk->code = NULL;
    initValueArray(&chunk->constants);
}

void writeChunk(Chunk *chunk, Opcode byte) {
    if (chunk->capacity < chunk->count + 1) {
        int oldCapacity = chunk->capacity;
        chunk->capacity = GROW_CAPACITY(oldCapacity);
        chunk->code = GROW_ARRAY(Opcode, chunk->code, oldCapacity, chunk->capacity);
    }
    chunk->code[chunk->count] = byte;
    chunk->count++;
}

void freeChunk(Chunk *chunk) {
    FREE_ARRAY(Opcode, chunk->code, chunk->capacity);
    initChunk(chunk);
}

int addConstant(Chunk *chunk, Value value) {
    writeValueArray(&chunk->constants, value);
    return chunk->constants.count - 1;
}