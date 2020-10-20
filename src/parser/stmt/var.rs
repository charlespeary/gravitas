use anyhow::{anyhow, Result};

use crate::parser::{Expr, Parser, Stmt, Token};

#[derive(Debug, Clone, PartialEq)]
pub struct VarStmt {
    pub expr: Expr,
    pub identifier: String,
}

impl Into<Stmt> for VarStmt {
    fn into(self) -> Stmt {
        Stmt::Var(self)
    }
}

impl Parser {
    pub fn parse_var_stmt(&mut self) -> Result<VarStmt> {
        let _token = self.next_token();
        if let Ok(identifier) = self.next_token().into_identifier() {
            self.expect(Token::Assign)?;
            let expr = self.parse_expr()?;
            self.expect(Token::Semicolon)?;
            Ok(VarStmt { expr, identifier })
        } else {
            Err(anyhow!("Something went wrong"))
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::{assert_eq, assert_ne};

    use crate::parser::expr::Atom;

    use super::*;

    #[test]
    fn var_stmt() {
        assert_eq!(
            Parser::new(vec![
                Token::Var,
                Token::Identifier(String::from("variable")),
                Token::Assign,
                Token::Number(10.0),
                Token::Semicolon,
            ])
            .parse_var_stmt()
            .unwrap(),
            VarStmt {
                identifier: String::from("variable"),
                expr: Expr::Atom(Atom::Number(10.0)),
            }
        );
    }
}
