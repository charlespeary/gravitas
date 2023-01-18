use bytecode::{chunk::Constant, stmt::GlobalPointer, MemoryAddress};
use common::{Number, ProgramText};

use crate::call::ObjectInstance;
use std::fmt;

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Number(Number),
    String(ProgramText),
    Bool(bool),
    ObjectInstance(ObjectInstance),
    MemoryAddress(MemoryAddress),
    Function(GlobalPointer),
    Class(GlobalPointer),
    // This will be an object instance of an Option in the future
    Null,
}

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use RuntimeValue::*;
        match self {
            Number(num) => write!(f, "{}", num),
            String(text) => write!(f, "{}", text),
            Bool(bool) => write!(f, "{}", bool),
            ObjectInstance(obj) => write!(f, "obj:{}", obj.class.name),
            MemoryAddress(address) => write!(f, "{}", address.to_string()),
            Null => write!(f, "null"),
            Function(ptr) => write!(f, "function"),
            Class(ptr) => write!(f, "class"),
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
            Constant::Function(ptr) => RuntimeValue::Function(ptr),
            Constant::Class(ptr) => RuntimeValue::Class(ptr),
            _ => unreachable!(),
        }
    }
}
