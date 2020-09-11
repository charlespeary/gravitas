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
    Unary {
        expr: Box<Expr>,
    },
    Grouping {
        expr: Box<Expr>,
    },
    Atom(Atom),
}

impl Expr {
    pub fn print(&self) {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                left.print();
                print!(" {} ", operator);
                right.print();
            }
            Expr::Unary { expr } => {
                print!("-(");
                expr.print();
                print!(")");
            }
            Expr::Atom(atom) => {
                print!("{}", atom);
            }
            Expr::Grouping { expr } => {
                print!("(");
                expr.print();
                print!(")");
            }
        }
    }
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
