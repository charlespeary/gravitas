use bytecode::{callables::Function, chunk::Constant};
use common::{Number, Symbol};

use crate::call::Callable;

#[derive(Debug, Clone)]
pub enum RuntimeValue {
    Number(Number),
    String(Symbol),
    Bool(bool),
    Callable(Callable),
}

impl From<Function> for RuntimeValue {
    fn from(fun: Function) -> RuntimeValue {
        RuntimeValue::Callable(Callable::Function(fun))
    }
}

impl From<Constant> for RuntimeValue {
    fn from(constant: Constant) -> Self {
        match constant {
            Constant::Number(num) => RuntimeValue::Number(num),
            Constant::String(str) => RuntimeValue::String(str),
            Constant::Bool(bl) => RuntimeValue::Bool(bl),
            Constant::Function(fun) => fun.into(),
        }
    }
}
