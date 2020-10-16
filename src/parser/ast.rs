use anyhow::Result;
use derive_more::Display;
use enum_as_inner::EnumAsInner;
use syn;

use proc_macro::TokenStream;
use quote::quote;

use crate::parser::{expr::Expr, stmt::Stmt};
use crate::utils::Either;

#[derive(Debug, PartialEq)]
pub struct Ast(pub Vec<Stmt>);

pub type ExprOrStmt = Either<Stmt, Expr>;

pub trait Visitor<T> {
    type Item;
    fn visit(&mut self, t: &T) -> Result<Self::Item>;
}

pub trait Visitable: Sized {
    fn accept<T>(&self, t: &mut T) -> Result<T::Item>
        where
            T: Visitor<Self>,
    {
        t.visit(self)
    }
}

impl Visitable for Ast {}


