use bytecode::stmt::GlobalPointer;

use crate::runtime_value::RuntimeValue;

pub(crate) type HeapPointer = usize;

#[derive(Debug)]
pub(crate) struct Closure {
    pub(crate) function_ptr: GlobalPointer,
    pub(crate) upvalues: Vec<RuntimeValue>,
}

impl Closure {
    pub fn new(function_ptr: GlobalPointer) -> Self {
        Self {
            function_ptr,
            upvalues: Vec::new(),
        }
    }

    pub fn close_upvalue(&mut self, value: RuntimeValue) {
        self.upvalues.push(value);
    }
}

#[derive(Debug)]
pub(crate) enum HeapObject {
    Closure(Closure),
}

impl HeapObject {
    pub fn as_closure(&self) -> &Closure {
        match self {
            Self::Closure(closure) => closure,
        }
    }
}

impl From<Closure> for HeapObject {
    fn from(closure: Closure) -> Self {
        Self::Closure(closure)
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
}
