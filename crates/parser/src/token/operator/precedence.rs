use crate::token::{Operator, Operator::*};

pub(crate) type InfixBindingPower = (u8, u8);
pub(crate) type PrefixBindingPower = ((), u8);
pub(crate) type PostfixBindingPower = (u8, ());

impl Operator {
    pub(crate) fn infix_bp(&self) -> Option<InfixBindingPower> {
        Some(match self {
            And | Or => (1, 2),
            Less | LessEqual | Greater | GreaterEqual | Compare | BangCompare => (3, 4),
            Plus | Minus => (5, 6),
            Multiply | Divide | Modulo => (7, 8),
            Exponent => (9, 10),
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
