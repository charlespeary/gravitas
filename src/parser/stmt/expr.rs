use anyhow::Result;

use crate::parser::{Expr, Parser, Stmt, Token};

pub(crate) struct ExprStmt {
    expr: Expr
}

impl Into<Stmt> for ExprStmt {
    fn into(self) -> Stmt {
        Stmt::Expr(self)
    }
}

impl Parser {
    pub fn parse_expr_stmt(&mut self) -> Result<ExprStmt> {
        let expr = self.expr()?;
        self.expect(Token::Semicolon)?;
        Ok(ExprStmt { expr })
    }
}