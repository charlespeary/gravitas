cmake_minimum_required(VERSION 3.17)
project(clox C)

set(CMAKE_C_STANDARD 11)

add_executable(clox main.c common.h chunk/chunk.h chunk/chunk.c utils/option.h memory/memory.h memory/memory.c disassembler/disassembler.h disassembler/disassembler.c value/value.h value/value.c)