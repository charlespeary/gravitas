use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::{
    bytecode::{Callable, Value},
    hashmap,
    std::functions::{clock, print},
};

mod functions;

pub type Args = Vec<Value>;

#[derive(Clone, PartialEq, PartialOrd)]
pub struct NativeFunction {
    pub arity: usize,
    pub function: fn(Args) -> Value,
}

impl Into<Value> for NativeFunction {
    fn into(self) -> Value {
        Value::Callable(Callable::NativeFunction(self))
    }
}

lazy_static! {
    pub static ref GLOBALS: HashMap<&'static str, NativeFunction> = hashmap! (
        "clock" => NativeFunction { arity: 0, function: clock },
        "print" => NativeFunction { arity: 1, function: print }
    );
}
