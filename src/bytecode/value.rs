use std::ops::Neg;

use anyhow::{anyhow, Result};

type Number = f64;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Null,
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

impl Value {
    pub fn as_number(self) -> Result<Number> {
        match self {
            Value::Number(num) => Ok(num),
            _ => Err(anyhow!("This isn't a number!")),
        }
    }
}

impl Into<bool> for Value {
    fn into(self) -> bool {
        match self {
            Value::Number(_) => true,
            Value::Null => false,
            Value::Bool(value) => value,
        }
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use super::*;

    // Value should return Ok(num) if it's a Value::Number
    #[quickcheck]
    fn ok_as_number(number: f64) -> Result<()> {
        Value::Number(number).as_number()?;
        Ok(())
    }

    // Value should return Err if it's not a Value::Number
    #[test]
    fn fail_as_number() {
        assert!(Value::Bool(false).as_number().is_err());
        assert!(Value::Bool(true).as_number().is_err());
        assert!(Value::Null.as_number().is_err());
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
}
