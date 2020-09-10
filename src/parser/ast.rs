use ordered_float::NotNan;

use crate::parser::Token;

#[derive(Debug)]
pub enum Atom {
    Text(String),
    Number(NotNan<f64>),
}

#[derive(Debug)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Atom(Atom),
}
