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
    chunk->lines = NULL;
    initValueArray(&chunk->constants);
}

void writeChunk(Chunk *chunk, Opcode byte, Line line) {
    if (chunk->capacity < chunk->count + 1) {
        int oldCapacity = chunk->capacity;
        chunk->capacity = GROW_CAPACITY(oldCapacity);
        chunk->code = GROW_ARRAY(Opcode, chunk->code, oldCapacity, chunk->capacity);
        chunk->lines = GROW_ARRAY(Line, chunk->lines, oldCapacity, chunk->capacity);
    }
    chunk->code[chunk->count] = byte;
    chunk->lines[chunk->count] = line;
    chunk->count++;
}

void freeChunk(Chunk *chunk) {
    FREE_ARRAY(Opcode, chunk->code, chunk->capacity);
    FREE_ARRAY(Line, chunk->lines, chunk->capacity);
    freeValueArray(&chunk->constants);
    initChunk(chunk);
}
//
//int addConstant(Chunk *chunk, Value value) {
//    writeValueArray(&chunk->constants, value);
//    return chunk->constants.count - 1;
//}

void writeConstant(Chunk *chunk, Value value, Line line) {
    int constantIndex = writeValueArray(&chunk->constants, value);

    if (true) {
        writeChunk(chunk, CONSTANT_LONG, line);
        writeChunk(chunk, (constantIndex >> 16) & 0xff, line);
        writeChunk(chunk, (constantIndex >> 8) & 0xff, line);
        writeChunk(chunk, constantIndex & 0xff, line);
    } else {
        writeChunk(chunk, CONSTANT, line);
        writeChunk(chunk, constantIndex, line);
    }

}

//int addLongConstant(Chunk *chunk, Value value) {
//    return chunk->constants.count - 1;
//}