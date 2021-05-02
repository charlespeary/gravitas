use crate::token::{Operator, Operator::*};

pub(crate) type BindingPower = (u8, u8);

impl Operator {
    pub(crate) fn bp(&self) -> BindingPower {
        match self {
            Plus | Minus => (1, 2),
            Multiply | Divide => (3, 4),
            _ => panic!("Operator binding power used in wrong context!"),
        }
    }
}
