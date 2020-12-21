use std::time::{SystemTime, UNIX_EPOCH};

use crate::{bytecode::Value, std::Args, utils::log};

// TODO: Maybe wrap it in some smart way to avoid repetition of injections

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
        println!("{}", arg);
    }
    Value::Null
}

// TEST

pub fn assert_eq(args: Args) -> Value {
    let a = args.get(0).unwrap();
    let b = args.get(1).unwrap();
    let message = format!("{} == {}", a, b);

    if a == b {
        log::success(&message);
    } else {
        log::error(&message);
    }
    Value::Null
}

pub fn assert(args: Args) -> Value {
    let callback = &args[0];
    let message = &args[1];
    println!("      {}", message.as_string().unwrap());
    Value::Null
}
