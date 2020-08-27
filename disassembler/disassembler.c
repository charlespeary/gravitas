//
// Created by karolgruszka on 27.08.2020.
//

#include <stdio.h>
#include "../chunk/chunk.h"

int simpleInstruction(const char *name, int offset) {
    printf("%s\n", name);
    return offset + 1;
}

static int constantInstruction(const char *name, Chunk *chunk, int offset) {
    Opcode constant = chunk->code[offset + 1];
    printf("%-16s %4d '", name, constant);
    printValue(chunk->constants.values[constant]);
    return offset + 2;
}

int disassembleInstruction(Chunk *chunk, int offset) {
    printf("%04d ", offset);

    uint8_t instruction = chunk->code[offset];

    switch (instruction) {
        case RETURN:
            return simpleInstruction("RETURN", offset);
        case CONSTANT:
            return constantInstruction("CONSTANT", chunk, offset);
        default:
            printf("Unknown instruction %d\n", instruction);
            return offset + 1;
    }
}

void disassembleChunk(Chunk *chunk, const char *name) {
    printf("== %s ==\n", name);
    for (int offset = 0; offset < chunk->count;) {
        printf("Going to disassemble... \n");
        // TODO: Debug why it stops there on constant instructions
        offset = disassembleInstruction(chunk, offset);
    }
}