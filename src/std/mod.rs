use std::cmp::Ordering;
use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::{
    bytecode::{Callable, Value},
    hashmap,
    std::functions::{assert_eq, clock, print, it},
    VM,
};

mod functions;

pub type Args = Vec<Value>;

#[derive(Clone)]
pub struct NativeFunction {
    pub arity: usize,
    pub function: fn(Args, &mut VM) -> Value,
}

impl std::cmp::PartialEq for NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

impl std::cmp::PartialOrd for NativeFunction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.arity.cmp(&other.arity))
    }
}

impl Into<Value> for NativeFunction {
    fn into(self) -> Value {
        Value::Callable(Callable::NativeFunction(self))
    }
}

lazy_static! {
    pub static ref GLOBALS: HashMap<&'static str, NativeFunction> = hashmap! (
        "clock" => NativeFunction { arity: 0, function: clock },
        "print" => NativeFunction { arity: 1, function: print },
        "assert_eq" => NativeFunction { arity: 2, function: assert_eq },
        "it" => NativeFunction { arity: 2, function: it }
    );
}
