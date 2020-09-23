use derive_more::Display;

use crate::parser::Token;

#[derive(Debug, Display, PartialEq)]
pub enum Atom {
    Text(String),
    Number(f64),
    Bool(bool),
    Null,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Var {
        identifier: String,
        is_ref: bool,
    },
    Unary {
        expr: Box<Expr>,
        operator: Token,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Block {
        body: Vec<Stmt>,
    },
    Atom(Atom),
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    // Expressions
    Expr { expr: Expr, terminated: bool },
    // Declarations
    Var { expr: Expr, identifier: String },
    // Class,
    // Func,
    // Side effects
    Print { expr: Expr },
}

pub trait Visitor<T> {
    type Result;
    fn visit(&mut self, t: &T) -> Self::Result;
}

pub trait Visitable: Sized {
    fn accept<T>(&self, t: &mut T) -> T::Result
    where
        T: Visitor<Self>,
    {
        t.visit(self)
    }
}

impl Visitable for Expr {}

impl Visitable for Stmt {}
