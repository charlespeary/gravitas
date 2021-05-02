use crate::token::{Operator, Operator::*};

pub(crate) type BindingPower = (u8, u8);

impl Operator {
    pub(crate) fn infix_bp(&self) -> BindingPower {
        match self {
            And | Or => (1, 2),
            Less | LessEqual | Greater | GreaterEqual | Compare | BangCompare => (3, 4),
            Plus | Minus => (5, 6),
            Multiply | Divide | Modulo => (7, 8),
            Exponent => (9, 10),

            _ => panic!("Operator binding power used in wrong context!"),
        }
    }
}
