use std::time::{SystemTime, UNIX_EPOCH};

use crate::{gravitas_std::FnArgs, runtime_value::RuntimeValue, VM};

pub fn clock(_: FnArgs, _: &mut VM) -> RuntimeValue {
    RuntimeValue::Number(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Can't access system time")
            .as_millis() as f64,
    )
}

pub fn print(args: FnArgs, _: &mut VM) -> RuntimeValue {
    for arg in args {
        println!("{}", arg);
    }
    RuntimeValue::Null
}
