use anyhow::Result;

use crate::parser::{
    stmt::{expr::ExprStmt, function::FunctionStmt, print::PrintStmt, var::VarStmt},
    Parser, Token,
};

use super::expr::{Binary, Block, Expr};

pub mod expr;
pub mod function;
pub mod print;
pub mod var;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(ExprStmt),
    Var(VarStmt),
    Print(PrintStmt),
    Function(FunctionStmt),
}

#[macro_export]
macro_rules! stmt {
    ($val: expr) => {
        Into::<Stmt>::into($val)
    };
}

#[macro_export]
macro_rules! try_stmt {
    ($val: expr) => {
        Into::<Stmt>::into($val?)
    };
}

impl Parser {
    pub fn stmt(&mut self) -> Result<Stmt> {
        Ok(match self.peek_token() {
            Token::Var => try_stmt!(self.parse_var_stmt()),
            Token::Function => try_stmt!(self.parse_function_stmt()),
            Token::Print => try_stmt!(self.parse_print_stmt()),
            _ => try_stmt!(self.parse_expr_stmt()),
        })
    }
}
