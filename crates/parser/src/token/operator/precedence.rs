use crate::token::{Operator, Operator::*};

pub(crate) type InfixBindingPower = (u8, u8);
pub(crate) type PrefixBindingPower = ((), u8);
pub(crate) type PostfixBindingPower = (u8, ());

impl Operator {
    pub(crate) fn infix_bp(&self) -> Option<InfixBindingPower> {
        Some(match self {
            Assign => (0, 1),
            And | Or => (2, 3),
            Less | LessEqual | Greater | GreaterEqual | Compare | BangCompare => (4, 5),
            Plus | Minus => (6, 7),
            Multiply | Divide | Modulo => (8, 9),
            Exponent => (10, 11),
            Dot => (12, 13),
            _ => return None,
        })
    }

    pub(crate) fn prefix_bp(&self) -> Option<PrefixBindingPower> {
        Some(match self {
            Plus | Minus | Bang => ((), 5),
            _ => return None,
        })
    }

    pub(crate) fn postfix_bp(&self) -> Option<PostfixBindingPower> {
        Some(match self {
            Operator::RoundBracketOpen | Operator::SquareBracketOpen => (11, ()),
            _ => return None,
        })
    }
}
