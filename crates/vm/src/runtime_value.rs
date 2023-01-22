use bytecode::{chunk::Constant, stmt::GlobalPointer, MemoryAddress};
use common::{Number, ProgramText};

use crate::gc::HeapPointer;
use std::fmt;

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Number(Number),
    String(ProgramText),
    Bool(bool),
    MemoryAddress(MemoryAddress),
    GlobalPointer(GlobalPointer),
    HeapPointer(HeapPointer),
    // This will be an object instance of an Option in the future
    Null,
}

impl RuntimeValue {
    pub fn as_pointer(self) -> GlobalPointer {
        match self {
            RuntimeValue::GlobalPointer(ptr) => ptr,
            x => panic!("Expected pointer, got {}", x),
        }
    }

    pub fn as_address(self) -> MemoryAddress {
        match self {
            RuntimeValue::MemoryAddress(address) => address,
            x => panic!("Expected address, got {}", x),
        }
    }
}

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use RuntimeValue::*;
        match self {
            Number(num) => write!(f, "{}", num),
            String(text) => write!(f, "{}", text),
            Bool(bool) => write!(f, "{}", bool),
            MemoryAddress(address) => write!(f, "{}", address.to_string()),
            Null => write!(f, "null"),
            GlobalPointer(ptr) => write!(f, "global ptr: {}", ptr),
            HeapPointer(ptr) => write!(f, "heap ptr: {}", ptr),
        }
    }
}

impl From<Constant> for RuntimeValue {
    fn from(constant: Constant) -> Self {
        match constant {
            Constant::Number(num) => RuntimeValue::Number(num),
            Constant::String(str) => RuntimeValue::String(str),
            Constant::Bool(bl) => RuntimeValue::Bool(bl),
            Constant::MemoryAddress(address) => RuntimeValue::MemoryAddress(address),
            Constant::GlobalPointer(ptr) => RuntimeValue::GlobalPointer(ptr),
        }
    }
}
