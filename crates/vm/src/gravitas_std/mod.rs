use std::{collections::HashMap, fmt};

use crate::{call::Callable, runtime_value::RuntimeValue, VM};
use common::ProgramText;
use lazy_static::lazy_static;

pub(crate) mod functions;
use functions::{clock, print};

pub(crate) type FnArgs = Vec<RuntimeValue>;
#[derive(Clone)]
pub struct BuiltInFunction {
    pub arity: usize,
    pub name: ProgramText,
    pub fn_body: fn(args: FnArgs, vm: &mut VM) -> RuntimeValue,
}

impl fmt::Debug for BuiltInFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BuiltInFunction")
            .field("arity", &self.arity)
            .field("name", &self.name)
            .field("fn_body", &"<built in function>")
            .finish()
    }
}

impl Into<RuntimeValue> for BuiltInFunction {
    fn into(self) -> RuntimeValue {
        RuntimeValue::Callable(Callable::BuiltInFunction(self))
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
    pub static ref STD_FUNCTIONS: HashMap<&'static str, BuiltInFunction> = hashmap! (
        "clock" => BuiltInFunction { arity: 0, fn_body: clock, name: "clock".to_owned() },
        "print" => BuiltInFunction { arity: 1, fn_body: print, name: "print".to_owned() }
    );
}
