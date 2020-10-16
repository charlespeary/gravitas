use crate::parser::{Expr, Parser, Stmt, Token};

pub(crate) struct VarStmt {
    expr: Expr,
    identifier: String,
}

impl Into<Stmt> for VarStmt {
    fn into(self) -> Stmt {
        Stmt::Var(self)
    }
}

impl Parser {
    pub fn parse_var_stmt(&mut self) {
        let _token = self.next_token();
        if let Ok(identifier) = self.next_token().into_identifier() {
            self.expect(Token::Assign)?;
            let expr = self.expr(0)?;
            self.expect(Token::Semicolon)?;
            Ok(VarStmt { expr, identifier })
        } else {
            Err(anyhow!("Something went wrong"))
        }
    }
}