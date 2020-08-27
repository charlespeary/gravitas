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
} Opcode;

typedef struct {
    int count;
    int capacity;
    Opcode *code;
    ValueArray constants;
} Chunk;

void initChunk(Chunk *chunk);

void writeChunk(Chunk *chunk, Opcode byte);

void freeChunk(Chunk *chunk);

int addConstant(Chunk *chunk, Value value);

#endif //CLOX_CHUNK_H
