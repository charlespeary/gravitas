use std::{collections::HashMap, fmt};

use crate::{runtime_value::RuntimeValue, VM};
use common::BuiltInFunction;
use lazy_static::lazy_static;

pub(crate) mod functions;
use functions::{clock, print};

pub(crate) type FnArgs = Vec<RuntimeValue>;
#[derive(Clone)]
pub struct NativeFunction {
    pub arity: usize,
    pub name: BuiltInFunction,
    pub fn_body: fn(args: FnArgs, vm: &mut VM) -> RuntimeValue,
}

impl fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeFunction")
            .field("arity", &self.arity)
            .field("name", &self.name)
            .field("fn_body", &"<built in function>")
            .finish()
    }
}

#[macro_export]
macro_rules! hashmap {
    ($($key:expr => $value:expr), *) => {{
        let mut hashmap = std::collections::HashMap::new();
        $(
            hashmap.insert($key, $value);
        )*
        hashmap
    }};
}

lazy_static! {
    pub static ref NATIVE_FUNCTIONS: HashMap<BuiltInFunction, NativeFunction> = hashmap! (
        BuiltInFunction::Clock => NativeFunction { arity: 0, fn_body: clock, name: BuiltInFunction::Clock },
        BuiltInFunction::Print => NativeFunction  { arity: 1, fn_body: print, name: BuiltInFunction::Print }
    );
}
