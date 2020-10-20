use anyhow::Result;
use enum_as_inner::EnumAsInner;

pub use crate::parser::expr::{
    atom::Atom,
    binary::{Binary, Operator},
    block::Block,
    conditional::If,
    grouping::Grouping,
    loops::{Break, Continue, WhileLoop},
    unary::Unary,
    var::Var,
};
use crate::parser::{Affix, Parser, Token};

pub mod atom;
pub mod binary;
pub mod block;
pub mod conditional;
pub mod grouping;
pub mod loops;
pub mod unary;
pub mod var;

#[derive(Debug, Clone, PartialEq, EnumAsInner)]
pub enum Expr {
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

#[macro_export]
macro_rules! expr {
    ($val: expr) => {
        Into::<Expr>::into($val)
    };
}

#[macro_export]
macro_rules! try_expr {
    ($val: expr) => {
        Into::<Expr>::into($val?)
    };
}

impl Parser {
    pub fn parse_expr(&mut self) -> Result<Expr> {
        Ok(self.parse_expr_bp(0)?)
    }

    pub fn parse_expr_bp(&mut self, rbp: usize) -> Result<Expr> {
        let mut expr = match self.peek_token() {
            Token::While => try_expr!(self.parse_while_loop()),
            Token::OpenParenthesis => try_expr!(self.parse_grouping()),
            Token::OpenBrace => try_expr!(self.parse_block()),
            Token::If => try_expr!(self.parse_if()),
            Token::Minus | Token::Bang => try_expr!(self.parse_unary()),
            Token::Identifier(_) => try_expr!(self.parse_var()),
            _ => try_expr!(self.parse_atom()),
        };

        while self.peek_bp(Affix::Infix) > rbp {
            expr = self.parse_binary(expr)?.into();
        }

        Ok(expr)
    }
}
