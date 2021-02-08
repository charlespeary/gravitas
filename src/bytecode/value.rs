use std::cmp::{Ordering, PartialOrd};
use std::fmt::Formatter;
use std::ops::{Add, Div, Mul, Sub};
use std::{fmt, ops::Neg};

use anyhow::{anyhow, Result};
use enum_as_inner::EnumAsInner;

use crate::bytecode::stmt::class::ObjectInstance;
use crate::{
    bytecode::expr::closure::Closure,
    bytecode::stmt::{class::Class, function::Function},
    std::NativeFunction,
};

pub type Number = f64;

#[derive(Debug, Clone, PartialEq, PartialOrd, EnumAsInner)]
pub enum Address {
    // Local variables, e.g defined inside block or a function.
    // This value is added to the function's stack offset.
    Local(usize),
    // Upvalue address
    // First value points to the stack index that starts at index
    // defined by callstack n (second value) jumps above.
    Upvalue(usize, usize),
    // Global variable refereed by a string key.
    // The value is extracted from a HashMap of globals.
    // Note: all of the variables and functions defined in vtas are "local" per se.
    // Only the std functions are global.
    Global(String),
}

impl Into<Value> for Address {
    fn into(self) -> Value {
        Value::Address(self)
    }
}

#[derive(Clone, PartialEq, PartialOrd, EnumAsInner)]
pub enum Callable {
    Function(Function),
    NativeFunction(NativeFunction),
    Closure(Closure),
    Class(Class),
}

impl fmt::Debug for Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Callable::Function(func) => f.write_fmt(format_args!("<fn {}>", func.name)),
            Callable::NativeFunction(_) => f.write_fmt(format_args!("<native fn>")),
            Callable::Closure(_) => f.write_fmt(format_args!("<closure fn>")),
            Callable::Class(class) => f.write_fmt(format_args!("<class {}>", class.name)),
        }
    }
}

impl Into<Value> for Callable {
    fn into(self) -> Value {
        Value::Callable(self)
    }
}

#[derive(Debug, Clone, PartialEq, EnumAsInner)]
pub enum Value {
    // Plain f64 number
    Number(Number),
    // Plain boolean value
    Bool(bool),
    // Plain String Value
    String(String),
    // Address, number that points to place on the stack (usize) or global variable (string)
    Address(Address),
    // Variant holding callable values
    Callable(Callable),
    // Array
    Array(Vec<Value>),
    // Null value, might be changed to Option in future
    Null,
    // Class instance
    Object(ObjectInstance),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(num) => write!(f, "{}", num),
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(str) => write!(f, "{}", str),
            Value::Address(address) => write!(f, "{:?}", address),
            Value::Callable(fun) => write!(f, "{:?}", fun),
            Value::Array(arr) => write!(f, "{:?}", arr),
            Value::Null => write!(f, "null"),
            Value::Object(obj) => write!(f, "<instance {}>", obj.class.name),
        }
    }
}

impl Neg for Value {
    type Output = Result<Value>;

    fn neg(self) -> Self::Output {
        match self {
            Value::Number(num) => Ok(Value::Number(-num)),
            _ => Err(anyhow!("Tried to negate value that can't be negated")),
        }
    }
}

macro_rules! implement_operations_for_value (
    ($($trait:ident $fn_name: ident,) *) => {
        $(
            impl $trait for Value {
                 type Output = Result<Value>;

                fn $fn_name(self, other: Self) -> Self::Output {
                    Ok(match (self, other) {
                        (Value::Number(a), Value::Number(b)) => Value::Number(std::ops::$trait::$fn_name(a,b)),
                        _ => {
                            return Err(anyhow!(
                                "Math operation on unsupported type!"
                            ))
                        }
                    })
                }
            }
        )*
    }
);

implement_operations_for_value!(
    Sub sub,
    Mul mul,
    Div div,
    Add add,

);

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl Into<bool> for Value {
    fn into(self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(value) => value,
            _ => true,
        }
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;
    use pretty_assertions::assert_eq;

    use super::*;

    // Value should return Ok(num) if it's a Value::Number
    #[quickcheck]
    fn ok_as_number(number: f64) {
        assert!(Value::Number(number).into_number().is_ok());
    }

    // Value should return Err if it's not a Value::Number
    #[test]
    fn fail_as_number() {
        assert!(Value::Bool(false).into_number().is_err());
        assert!(Value::Bool(true).into_number().is_err());
        assert!(Value::Null.into_number().is_err());
    }

    // Number values can be negated via unary - operator, but unsupported values should fail
    #[quickcheck]
    fn negate_value(value: f64) -> Result<()> {
        Value::Number(value).neg()?;
        Ok(())
    }

    // These values can't be negated with - so we throw an error.
    #[test]
    fn fail_value_negation() {
        assert!(Value::Bool(false).neg().is_err());
        assert!(Value::Bool(true).neg().is_err());
        assert!(Value::Null.neg().is_err());
        assert!(Value::Address(Address::Global(String::from("var")))
            .neg()
            .is_err());
        assert!(Value::Address(Address::Local(0)).neg().is_err());
    }

    // Value can be turned into bool. This acts as a helper to determine whether value is falsey or not.
    // Everything except Null and Value::Bool(false) is truthy.
    #[test]
    fn standard_values_into_bool() {
        // We don't treat 0 or negative numbers as falsy values.
        assert_eq!(true, Value::Number(0.0).into());
        assert_eq!(true, Value::Number(-1.0).into());
        assert_eq!(true, Value::Bool(true).into());
        assert_eq!(false, Value::Bool(false).into());
        assert_eq!(false, Value::Null.into());
    }

    #[quickcheck]
    fn random_number_values_into_bool(value: f64) {
        assert_eq!(true, Value::Number(value).into())
    }

    macro_rules! math_op (
        ($a: expr, $b: expr, $operator: tt ) => {{
                let first = Value::Number($a);
                let second = Value::Number($b);
                (first $operator second).unwrap_or_else(|_| {
                panic!(
                    "Unexpectedly failed the math operation between {} and {}",
                    $a, $b
                )
            })
        }}
    );

    #[quickcheck]
    fn add_number_values(a: f64, b: f64) {
        let sum = math_op!(a, b, +);
        assert_eq!(sum, Value::Number(a + b));
    }

    #[quickcheck]
    fn subtract_number_values(a: f64, b: f64) {
        let subtraction = math_op!(a, b, -);
        assert_eq!(subtraction, Value::Number(a - b));
    }

    #[quickcheck]
    fn multiply_number_values(a: f64, b: f64) {
        let multiplication = math_op!(a, b, *);
        assert_eq!(multiplication, Value::Number(a * b));
    }

    #[quickcheck]
    fn divide_number_values(a: f64, b: f64) {
        let division = math_op!(a, b, /);
        assert_eq!(division, Value::Number(a / b));
    }
}
