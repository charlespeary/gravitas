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

// impl PartialOrd for Value {
//     fn partial_cmp(self, other: &Value) ->
// }
