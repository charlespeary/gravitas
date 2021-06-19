use bytecode::chunk::Constant;
use common::{Number, Symbol};

#[derive(Debug, Clone, Copy)]
pub enum RuntimeValue {
    Number(Number),
    String(Symbol),
    Bool(bool),
}

impl From<Constant> for RuntimeValue {
    fn from(constant: Constant) -> Self {
        match constant {
            Constant::Number(num) => RuntimeValue::Number(num),
            Constant::String(str) => RuntimeValue::String(str),
            Constant::Bool(bl) => RuntimeValue::Bool(bl),
        }
    }
}
