use crate::token::{Operator, Operator::*};

pub(crate) type InfixBindingPower = (u8, u8);
pub(crate) type PrefixBindingPower = ((), u8);

impl Operator {
    pub(crate) fn infix_bp(&self) -> InfixBindingPower {
        match self {
            And | Or => (1, 2),
            Less | LessEqual | Greater | GreaterEqual | Compare | BangCompare => (3, 4),
            Plus | Minus => (5, 6),
            Multiply | Divide | Modulo => (7, 8),
            Exponent => (9, 10),
            _ => panic!("{} doesn't support infix binding power!", self),
        }
    }

    pub(crate) fn prefix_bp(&self) -> PrefixBindingPower {
        match self {
            Plus | Minus | Bang => ((), 5),
            _ => panic!("{} doesn't support prefix binding power!", self),
        }
    }
}
