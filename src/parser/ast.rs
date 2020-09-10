use ordered_float::NotNan;

use crate::parser::Token;

#[derive(Debug, Eq, PartialEq)]
pub enum Atom {
    Text(String),
    Number(NotNan<f64>),
}

#[derive(Debug, Eq, PartialEq)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        expr: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Atom(Atom),
}
