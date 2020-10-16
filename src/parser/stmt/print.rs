use anyhow::Result;

use crate::parser::{expr::Expr, Parser, stmt::Stmt, Token};

pub(crate) struct PrintStmt { expr: Expr }

impl Parser {
    pub fn parse_print_stmt(&mut self) -> Result<Stmt> {
        self.expect(Token::Print)?;
        let expr = self.parse_expr()?;
        self.expect(Token::Semicolon)?;
        Ok(PrintStmt { expr })
    }
}