//
// Created by karolgruszka on 27.08.2020.
//

#ifndef CLOX_DISASSEMBLER_H
#define CLOX_DISASSEMBLER_H

#include "../chunk/chunk.h"

void disassembleChunk(Chunk *chunk, const char *name);

void disassembleInstruction(Chunk *chunk, int offset);

#endif //CLOX_DISASSEMBLER_H
