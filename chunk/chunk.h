//
// Created by karolgruszka on 27.08.2020.
//

#ifndef CLOX_CHUNK_H
#define CLOX_CHUNK_H

#include "../common.h"
#include "../value/value.h"


typedef enum {
    RETURN,
    CONSTANT,
    CONSTANT_LONG
} Opcode;

// TODO: Challenge 1:1
typedef int Line;

typedef struct {
    int count;
    int capacity;
    uint8_t *code;
    Line *lines;
    ValueArray constants;
} Chunk;

void initChunk(Chunk *chunk);

void writeChunk(Chunk *chunk, Opcode byte, Line line);

void freeChunk(Chunk *chunk);

void writeConstant(Chunk *chunk, Value value, Line line);

#endif //CLOX_CHUNK_H
