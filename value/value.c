//
// Created by karolgruszka on 27.08.2020.
//


#include <stdio.h>
#include "value.h"
#include "../memory/memory.h"

void initValueArray(ValueArray *array) {
    array->values = NULL;
    array->capacity = 0;
    array->count = 0;
}

int writeValueArray(ValueArray *array, Value value) {
    if (array->capacity < array->count + 1) {
        int oldCapacity = array->capacity;
        array->capacity = GROW_CAPACITY(oldCapacity);
        array->values = GROW_ARRAY(Value, array->values, oldCapacity, array->capacity);
    }
    array->values[array->count] = value;
    array->count++;
    return array->count - 1;
}

void freeValueArray(ValueArray *array) {
    FREE_ARRAY(Value, array->values, array->capacity);
    initValueArray(array);
}

void printValue(Value value) {
    printf("%g", value);
}
