use logos::Lexer;

use crate::common::error::ParseErrorCause;
use crate::token::Token;
use std::str::FromStr;

pub(crate) mod precedence;

pub(crate) static OPERATORS: &[&str] = &[
    "+", "-", "*", "/", "%", "**", "=", "==", "!=", "<", "<=", ">", ">=", "or", "and", "!", ".",
    "[", "]", "(", ")",
];

pub(crate) static BINARY_OPERATORS: &[&str] = &[
    "+", "-", "*", "/", "%", "**", "==", "!=", "<", "<=", ">", ">=", "or", "and",
];

pub(crate) static UNARY_OPERATORS: &[&str] = &["!", "-"];
pub(crate) static POSTFIX_OPERATORS: &[&'static str] = &[".", "[", "]", "(", ")"];

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Exponent,
    Compare,
    BangCompare,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Or,
    And,
    Bang,
    Assign,
    Dot,
    RoundBracketOpen,
    RoundBracketClose,
    SquareBracketOpen,
    SquareBracketClose,
    CurlyBracketOpen,
    CurlyBracketClose,
}

macro_rules! impl_from_to_str {
    ($($op_str: literal => $op: path),*) => {
        impl std::str::FromStr for Operator {
            type Err = ParseErrorCause;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(match s {
                    $($op_str => $op),*,
                    _ => return Err(ParseErrorCause::UnexpectedToken),
                })
            }
        }

        impl std::fmt::Display for Operator {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let s = match self {
                    $($op => $op_str),*
                };
                f.write_str(s)
            }
        }
    };
}

impl_from_to_str!(
    "+" => Operator::Plus,
    "-" => Operator::Minus,
    "*" => Operator::Multiply,
    "/" => Operator::Divide,
    "%" => Operator::Modulo,
    "**" => Operator::Exponent,
    "=" => Operator::Assign,
    "==" => Operator::Compare,
    "!=" => Operator::BangCompare,
    "<" => Operator::Less,
    "<=" => Operator::LessEqual,
    ">" => Operator::Greater,
    ">=" => Operator::GreaterEqual,
    "or" => Operator::Or,
    "and" => Operator::And,
    "!" => Operator::Bang,
    "." => Operator::Dot,
    "[" => Operator::SquareBracketOpen,
    "]" => Operator::SquareBracketClose,
    "(" => Operator::RoundBracketOpen,
    ")" => Operator::RoundBracketClose,
    "{" => Operator::CurlyBracketOpen,
    "}" => Operator::CurlyBracketClose
);

pub(crate) fn lex_operator<'t>(lex: &mut Lexer<'t, Token<'t>>) -> Option<Operator> {
    let slice: String = lex.slice().parse().ok()?;
    Operator::from_str(slice.as_str()).ok()
}

#[cfg(test)]
#[macro_use]
mod test {
    use crate::{
        common::test::{assert_token, op},
        token::{operator::Operator, Token},
    };

    #[test]
    fn lex_all_operators() {
        use Operator::*;
        assert_token("+", op(Plus));
        assert_token("-", op(Minus));
        assert_token("*", op(Multiply));
        assert_token("/", op(Divide));
        assert_token("%", op(Modulo));
        assert_token("**", op(Exponent));
        assert_token("=", op(Assign));
        assert_token("==", op(Compare));
        assert_token("!=", op(BangCompare));
        assert_token("<", op(Less));
        assert_token("<=", op(LessEqual));
        assert_token(">", op(Greater));
        assert_token(">=", op(GreaterEqual));
        assert_token("or", op(Or));
        assert_token("and", op(And));
        assert_token("!", op(Bang));
        assert_token(".", op(Dot));
        assert_token("(", op(RoundBracketOpen));
        assert_token(")", op(RoundBracketClose));
        assert_token("[", op(SquareBracketOpen));
        assert_token("]", op(SquareBracketClose));
        assert_token("{", op(CurlyBracketOpen));
        assert_token("}", op(CurlyBracketClose));
    }
}
