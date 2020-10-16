use anyhow::Result;
use enum_as_inner::EnumAsInner;

use crate::parser::{Affix, ast::Visitable, IfBranch, Parser, Token};

pub use self::{
    atom::Atom,
    binary::{Binary, Operator},
    block::Block,
    conditional::If,
    grouping::Grouping,
    loops::{Break, Continue, WhileLoop},
    unary::Unary,
    var::Var,
};

pub mod atom;
pub mod binary;
pub mod block;
pub mod loops;
pub mod conditional;
pub mod unary;
pub mod grouping;

#[derive(Debug, PartialEq, EnumAsInner)]
pub(crate) enum Expr {
    Binary(Binary),
    Var(Var),
    Unary(Unary),
    Grouping(Grouping),
    Block(Block),
    If(If),
    While(WhileLoop),
    Break(Break),
    Continue(Continue),
    Atom(Atom),
}

impl Visitable for Expr {}

impl Parser {
    pub fn parse_expr(&mut self) -> Result<Expr> {
        Ok(self.parse_expr_bp(0)?)
    }

    pub fn parse_expr_bp(&mut self, rbp: usize) -> Result<Expr> {
        let mut expr: Expr = match self.next_token() {
            _ => self.parse_atom()
        }.into();

        while self.peek_bp(Affix::Infix) > rbp {
            expr = self.parse_binary(expr, 0)?.into();
        }
    }
}