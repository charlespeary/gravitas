use std::time::{SystemTime, UNIX_EPOCH};

use crate::{bytecode::Value, std::Args};

pub fn clock(_: Args) -> Value {
    Value::Number(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Can't access system time")
            .as_millis() as f64,
    )
}

pub fn print(args: Args) -> Value {
    for arg in args {
        println!("{:?}", arg);
    }
    Value::Null
}
