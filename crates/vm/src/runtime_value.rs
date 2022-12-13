use bytecode::{
    callables::{Class, Function},
    chunk::Constant,
};
use common::{Number, ProgramText};

use crate::call::{Callable, ObjectInstance};

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Number(Number),
    String(ProgramText),
    Bool(bool),
    Callable(Callable),
    ObjectInstance(ObjectInstance),
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
            _ => panic!("Tried to convert invalid value"),
        }
    }
}
