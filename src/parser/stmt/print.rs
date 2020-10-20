use anyhow::Result;

use crate::parser::{expr::Expr, stmt::Stmt, Parser, Token};

#[derive(Debug, Clone, PartialEq)]
pub struct PrintStmt {
    pub expr: Expr,
}

impl Into<Stmt> for PrintStmt {
    fn into(self) -> Stmt {
        Stmt::Print(self)
    }
}

impl Parser {
    pub fn parse_print_stmt(&mut self) -> Result<PrintStmt> {
        self.expect(Token::Print)?;
        let expr = self.parse_expr()?;
        self.expect(Token::Semicolon)?;
        Ok(PrintStmt { expr })
    }
}
