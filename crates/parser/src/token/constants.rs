use crate::token::{operator::Operator, Token};

pub(crate) const OPERATORS: &[&str] = &[
    "+", "-", "*", "/", "%", "**", "=", "==", "!=", "<", "<=", ">", ">=", "or", "and", "!", ".",
    "[", "]", "(", ")",
];
pub(crate) const BINARY_OPERATORS: &[&str] = &[
    "+", "-", "*", "/", "%", "**", "==", "!=", "<", "<=", ">", ">=", "or", "and",
];
pub(crate) const UNARY_OPERATORS: &[&str] = &["!", "-"];
pub(crate) const POSTFIX_OPERATORS: &[&str] = &[".", "[", "]", "(", ")"];
// Dummy constants to use for discriminant comparisons
pub(crate) const IDENTIFIER: Token = Token::Identifier("");
pub(crate) const OPERATOR: Token = Token::Operator(Operator::Assign);

pub(crate) const OPEN_BRACKET: Token = Token::Operator(Operator::CurlyBracketOpen);
pub(crate) const CLOSE_BRACKET: Token = Token::Operator(Operator::CurlyBracketClose);

pub(crate) const OPEN_PARENTHESIS: Token = Token::Operator(Operator::RoundBracketOpen);
pub(crate) const CLOSE_PARENTHESIS: Token = Token::Operator(Operator::RoundBracketClose);

pub(crate) const OPEN_SQUARE: Token = Token::Operator(Operator::SquareBracketOpen);
pub(crate) const CLOSE_SQUARE: Token = Token::Operator(Operator::SquareBracketClose);

pub(crate) const DOT: Token = Token::Operator(Operator::Dot);
