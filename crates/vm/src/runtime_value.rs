use bytecode::{
    callables::{Class, Function},
    chunk::Constant,
    MemoryAddress,
};
use common::{Number, ProgramText};

use crate::call::{Callable, ObjectInstance};
use std::fmt;

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Number(Number),
    String(ProgramText),
    Bool(bool),
    Callable(Callable),
    ObjectInstance(ObjectInstance),
    MemoryAddress(MemoryAddress),
    // Temporary placeholder
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
            Callable(callable) => write!(f, "callable"),
            ObjectInstance(obj) => write!(f, "obj:{}", obj.class.name),
            MemoryAddress(address) => write!(f, "address"),
            Null => write!(f, "null"),
        }
    }
}

impl From<Function> for RuntimeValue {
    fn from(fun: Function) -> RuntimeValue {
        RuntimeValue::Callable(Callable::Function(fun))
    }
}

impl From<Class> for RuntimeValue {
    fn from(class: Class) -> RuntimeValue {
        RuntimeValue::Callable(Callable::Class(class))
    }
}

impl From<Constant> for RuntimeValue {
    fn from(constant: Constant) -> Self {
        match constant {
            Constant::Number(num) => RuntimeValue::Number(num),
            Constant::String(str) => RuntimeValue::String(str),
            Constant::Bool(bl) => RuntimeValue::Bool(bl),
            Constant::Function(fun) => fun.into(),
            Constant::Class(class) => class.into(),
            Constant::MemoryAddress(address) => RuntimeValue::MemoryAddress(address),
        }
    }
}
