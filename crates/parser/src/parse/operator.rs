use crate::{common::error::ParseErrorCause, token::operator::Operator};
use std::convert::TryInto;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum BinaryOperator {
    // +
    Add,
    // -
    Sub,
    // *
    Mul,
    // /
    Div,
    // %
    Mod,
    // **
    Exp,
    // ==
    Eq,
    // !=
    Ne,
    // <
    Lt,
    // <=
    Le,
    // >
    Gt,
    // >=
    Ge,
    // or
    Or,
    // and
    And,
}

impl FromStr for BinaryOperator {
    type Err = ParseErrorCause;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Operator::from_str(s)?.try_into()
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Operator::from(*self))
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum UnaryOperator {
    Negate,
    Not,
}

impl From<Operator> for UnaryOperator {
    fn from(operator: Operator) -> Self {
        match operator {
            Operator::Bang => UnaryOperator::Not,
            Operator::Minus => UnaryOperator::Negate,
            _ => unreachable!(),
        }
    }
}

macro_rules! impl_double_ended_conversion {
    ($($operator: path => $binary_operator: path),*) => {
        impl std::convert::TryFrom<Operator> for BinaryOperator {
            type Error = ParseErrorCause;

            fn try_from(op: Operator) -> Result<Self, Self::Error> {
                Ok(match op {
                    $($operator => $binary_operator),*,
                    _ => return Err(ParseErrorCause::UnexpectedToken),
                })
            }
        }

        impl std::convert::From<BinaryOperator> for Operator {

            fn from(op: BinaryOperator) -> Self {
                match op {
                    $($binary_operator => $operator),*,
                }
            }
        }
    };
}

impl_double_ended_conversion!(
    Operator::Plus => BinaryOperator::Add,
    Operator::Minus => BinaryOperator::Sub,
    Operator::Multiply => BinaryOperator::Mul,
    Operator::Divide => BinaryOperator::Div,
    Operator::Modulo => BinaryOperator::Mod,
    Operator::Exponent => BinaryOperator::Exp,
    Operator::Compare => BinaryOperator::Eq,
    Operator::BangCompare => BinaryOperator::Ne,
    Operator::Less => BinaryOperator::Lt,
    Operator::LessEqual => BinaryOperator::Le,
    Operator::Greater => BinaryOperator::Gt,
    Operator::GreaterEqual => BinaryOperator::Ge,
    Operator::Or => BinaryOperator::Or,
    Operator::And => BinaryOperator::And
);

#[cfg(test)]
mod test {
    use super::*;
    use crate::token::operator::{Operator, BINARY_OPERATORS};
    use quickcheck::{Arbitrary, Gen};
    use std::convert::TryFrom;

    impl Arbitrary for BinaryOperator {
        fn arbitrary(g: &mut Gen) -> Self {
            BinaryOperator::try_from(
                Operator::from_str(g.choose(&BINARY_OPERATORS).unwrap()).unwrap(),
            )
            .unwrap()
        }
    }
}
