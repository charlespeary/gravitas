use crate::token::Token;
use lazy_static::lazy_static;
use std::collections::HashMap;

pub(crate) static OPERATORS: &[&str] = &[
    "+", "-", "*", "/", "%", "**", "=", "==", "!=", "<", "<=", ">", ">=", "or", "and", "!", ".",
    "[", "]", "(", ")",
];
pub(crate) static BINARY_OPERATORS: &[&str] = &[
    "+", "-", "*", "/", "%", "**", "==", "!=", "<", "<=", ">", ">=", "or", "and",
];
pub(crate) static UNARY_OPERATORS: &[&str] = &["!", "-"];
pub(crate) static POSTFIX_OPERATORS: &[&str] = &[".", "[", "]", "(", ")"];
