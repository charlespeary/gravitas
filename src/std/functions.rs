use std::time::{SystemTime, UNIX_EPOCH};

use crate::{bytecode::Value, std::Args, utils::log, VM};

pub fn clock(_: Args, _: &mut VM) -> Value {
    Value::Number(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Can't access system time")
            .as_millis() as f64,
    )
}

pub fn print(args: Args, _: &mut VM) -> Value {
    for arg in args {
        println!("{}", arg);
    }
    Value::Null
}

// TESTING FUNCTIONS

pub fn assert_eq(args: Args, vm: &mut VM) -> Value {
    let a = args.get(0).unwrap();
    let b = args.get(1).unwrap();
    let message = format!("{} == {}", a, b);

    let test_runner = vm.injections.get_test_runner();
    test_runner.run();

    if a == b {
        log::success_indent(&message, 1);
        test_runner.success();
    } else {
        log::error_indent(&message, 1);
        test_runner.failure();
    }
    Value::Null
}
