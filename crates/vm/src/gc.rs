use core::panic;

use bytecode::stmt::GlobalPointer;

use crate::{call::ObjectInstance, runtime_value::RuntimeValue};

pub(crate) type HeapPointer = usize;

#[derive(Debug)]
pub(crate) struct Closure {
    pub(crate) function_ptr: GlobalPointer,
    pub(crate) upvalues: Vec<HeapPointer>,
}

impl Closure {
    pub fn new(function_ptr: GlobalPointer) -> Self {
        Self {
            function_ptr,
            upvalues: Vec::new(),
        }
    }

    pub fn close_upvalue(&mut self, upvalue_ptr: HeapPointer) {
        self.upvalues.push(upvalue_ptr);
    }
}

#[derive(Debug)]
pub(crate) enum HeapObject {
    Closure(Closure),
    Object(ObjectInstance),
    Value(RuntimeValue),
}

impl HeapObject {
    pub fn as_closure(&self) -> &Closure {
        match self {
            Self::Closure(closure) => closure,
            _ => panic!("Expected closure"),
        }
    }

    pub fn as_value(&self) -> &RuntimeValue {
        match self {
            Self::Value(value) => value,
            _ => panic!("Expected value"),
        }
    }
}

impl From<Closure> for HeapObject {
    fn from(closure: Closure) -> Self {
        Self::Closure(closure)
    }
}

impl From<RuntimeValue> for HeapObject {
    fn from(value: RuntimeValue) -> Self {
        Self::Value(value)
    }
}

#[derive(Debug)]
pub(crate) struct GC {
    objects: Vec<HeapObject>,
}

impl GC {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn allocate(&mut self, object: HeapObject) -> HeapPointer {
        self.objects.push(object);
        self.objects.len() - 1
    }

    pub fn deref(&self, pointer: HeapPointer) -> &HeapObject {
        self.objects.get(pointer).unwrap()
    }

    pub fn deref_mut(&mut self, pointer: HeapPointer) -> &mut HeapObject {
        self.objects.get_mut(pointer).unwrap()
    }
}
