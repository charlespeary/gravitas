use anyhow::Result;

use crate::parser::{
    Parser,
    stmt::{expr::ExprStmt, function::FunctionStmt, var::VarStmt}, Token,
};

pub mod expr;
pub mod function;
pub mod var;

/// Statements are used to perform side effects, such as kicking off the expression's evaluation or
/// variable, class, function declarations.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr(ExprStmt),
    Var(VarStmt),
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
    /// Looks for statement's keyword and tries to parse appropriate part of grammar.
    pub fn stmt(&mut self) -> Result<Stmt> {
        Ok(match self.peek_token()? {
            Token::Var => try_stmt!(self.parse_var_stmt()),
            Token::Function => try_stmt!(self.parse_function_stmt()),
            _ => try_stmt!(self.parse_expr_stmt()),
        })
    }
}
