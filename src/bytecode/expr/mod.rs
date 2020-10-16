use anyhow::Result;

use crate::{BytecodeGenerator, parser::{ast::Visitor, expr::Expr, stmt::Stmt}};
use crate::parser::ast::Visitable;

mod atom;
mod binary;
mod block;
mod conditional;
mod grouping;
mod loops;
mod unary;
mod var;

impl Visitor<Expr> for BytecodeGenerator {
    type Item = Result<()>;

    fn visit(&mut self, expr: &Expr) -> Self::Item {
        match expr {
            Expr::Block(block) => block.accept(self),
            Expr::Var(var) => var.accept(self),
            Expr::Continue(con) => con.accept(self),
            Expr::Break(bre) => bre.accept(self),
            Expr::Grouping(group) => group.accept(self),
            Expr::While(wl) => wl.accept(self),
            Expr::Atom(atom) => atom.accept(self),
            Expr::Unary(unary) => unary.accept(self),
            Expr::If(ifc) => ifc.accept(self),
            Expr::Binary(binary) => binary.accept(self)
        }
    }
}