use crate::{common::error::ParseErrorCause, token::operator::Operator};
use std::convert::TryInto;
use std::fmt;
use std::str::FromStr;

macro_rules! impl_double_ended_conversion {
    ($from: ty, $to: ty, [$($operator: path => $binary_operator: path),*]) => {
        impl std::convert::TryFrom<$from> for $to {
            type Error = ParseErrorCause;

            fn try_from(op: $from) -> Result<Self, Self::Error> {
                Ok(match op {
                    $($operator => $binary_operator),*,
                    _ => return Err(ParseErrorCause::UnexpectedToken),
                })
            }
        }

        impl std::convert::From<$to> for $from {

            fn from(op: $to) -> Self {
                match op {
                    $($binary_operator => $operator),*,
                }
            }
        }
    };
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum BinaryOperator {
    // +
    Addition,
    // -
    Subtraction,
    // *
    Multiplication,
    // /
    Division,
    // %
    Modulo,
    // **
    Power,
    // ==
    Equals,
    // !=
    NotEquals,
    // <
    LesserThan,
    // <=
    LesserEquals,
    // >
    GreaterThan,
    // >=
    GreaterEquals,
    // or
    Or,
    // and
    And,
}

impl_double_ended_conversion!(
    Operator, BinaryOperator, [
        Operator::Plus => BinaryOperator::Addition,
        Operator::Minus => BinaryOperator::Subtraction,
        Operator::Multiply => BinaryOperator::Multiplication,
        Operator::Divide => BinaryOperator::Division,
        Operator::Modulo => BinaryOperator::Modulo,
        Operator::Exponent => BinaryOperator::Power,
        Operator::Compare => BinaryOperator::Equals,
        Operator::BangCompare => BinaryOperator::NotEquals,
        Operator::Less => BinaryOperator::LesserThan,
        Operator::LessEqual => BinaryOperator::LesserEquals,
        Operator::Greater => BinaryOperator::GreaterThan,
        Operator::GreaterEqual => BinaryOperator::GreaterEquals,
        Operator::Or => BinaryOperator::Or,
        Operator::And => BinaryOperator::And
    ]
);

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

impl_double_ended_conversion!(
    Operator, UnaryOperator, [
        Operator::Minus => UnaryOperator::Negate,
        Operator::Bang => UnaryOperator::Not
    ]
);

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Operator::from(*self))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::token::{constants::BINARY_OPERATORS, operator::Operator};
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
