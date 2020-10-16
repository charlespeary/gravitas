use anyhow::Result;

use crate::parser::{ast::Visitable, Parser, stmt::{expr::ExprStmt, function::FunctionStmt, print::PrintStmt, var::VarStmt}, Token};

use super::expr::{Binary, Block, Expr};

pub mod expr;
pub mod function;
pub mod var;
pub mod print;

#[derive(Debug, PartialEq)]
pub(crate) enum Stmt {
    Expr(ExprStmt),
    Var(VarStmt),
    Print(PrintStmt),
    Function(FunctionStmt),
}

impl Visitable for Stmt {}

impl Parser {
    pub fn stmt(&mut self) -> Result<Stmt> {
        match self.peek_token() {
            Token::Var => self.parse_var_stmt(),
            Token::Function => self.parse_function_stmt(),
            Token::Print => self.parse_print_stmt(),
            _ => self.parse_expr_stmt()
        }
    }
}