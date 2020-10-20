use anyhow::Result;

use crate::parser::{Expr, Parser, Stmt, Token};

#[derive(Debug, Clone, PartialEq)]
pub struct ExprStmt {
    pub expr: Expr,
}

impl Into<Stmt> for ExprStmt {
    fn into(self) -> Stmt {
        Stmt::Expr(self)
    }
}

impl Parser {
    pub fn parse_expr_stmt(&mut self) -> Result<ExprStmt> {
        let expr = self.parse_expr()?;
        self.expect(Token::Semicolon)?;
        Ok(ExprStmt { expr })
    }
}
