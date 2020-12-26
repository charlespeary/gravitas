use std::time::{SystemTime, UNIX_EPOCH};

use crate::{bytecode::{Callable, Value}, std::Args, utils::logger, VM};
use crate::utils::logger::log;

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
    // Lookup if test runner is available in VM
    // If it isn't then we're not running in test mode and false assertion should panic the application
    match vm.utilities.as_mut().map(|u| u.test_runner.as_mut()).flatten() {
        Some(test_runner) => {
            test_runner.run();
            if a == b {
                log(&message).success().indent(3);
                test_runner.success();
            } else {
                log(&message).error().indent(3);
                test_runner.failure();
            }
        }
        None => {
            if !(a == b) {
                panic!(format!("Assertion failed at {} == {}", a, b))
            }
        }
    }

    Value::Null
}

pub fn it(args: Args, vm: &mut VM) -> Value {
    log(format!("{}", args[1]).as_str()).indent(2);
    let callback = args[0].clone().into_callable().unwrap();
    vm.callback(callback);
    Value::Null
}
