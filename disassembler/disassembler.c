//
// Created by karolgruszka on 27.08.2020.
//

#include <stdio.h>
#include "../chunk/chunk.h"

int simpleInstruction(const char *name, int offset) {
    printf("%s\n", name);
    return offset + 1;
}

void printLine(Line line) {
    printf(" at line: %d", line);
}

static int constantInstruction(Chunk *chunk, int offset) {
    Opcode constant = chunk->code[offset + 1];
    printf("%-16s", "CONSTANT");
    printValue(chunk->constants.values[constant]);
    printLine(chunk->lines[offset + 1]);
    return offset + 2;
}

static int constantLongInstruction(Chunk *chunk, int offset) {
    printf("%-16s", "CONSTANT_LONG");
    uint8_t first = chunk->code[offset + 1];
    uint8_t second = chunk->code[offset + 2];
    uint8_t third = chunk->code[offset + 3];
    uint32_t index = first;
    index = (index << 8) | second;
    index = (index << 8) | third;
    printValue(chunk->constants.values[index]);
    return offset + 4;
}

int disassembleInstruction(Chunk *chunk, int offset) {
    printf("%04d ", offset);

    if (offset > 0 && chunk->lines[offset] == chunk->lines[offset - 1]) {
        printf("   | ");
    } else {
        printf("%4d ", chunk->lines[offset]);
    }

    Opcode instruction = chunk->code[offset];

    switch (instruction) {
        case RETURN:
            return simpleInstruction("RETURN", offset);
        case CONSTANT:
            return constantInstruction(chunk, offset);
        case CONSTANT_LONG:
            return constantLongInstruction(chunk, offset);
        default:
            printf("Unknown instruction %d\n", instruction);
            return offset + 1;
    }
}

void disassembleChunk(Chunk *chunk, const char *name) {
    printf("== %s ==\n", name);
    for (int offset = 0; offset < chunk->count;) {
        offset = disassembleInstruction(chunk, offset);
    }
}