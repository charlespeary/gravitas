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
