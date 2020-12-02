use anyhow::{anyhow, Result};

use crate::parser::{Expr, operator::Operator, Parser, Stmt, Token};

///
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
    /// Parse variable declaration statement
    pub fn parse_var_stmt(&mut self) -> Result<VarStmt> {
        let _token = self.next_token();
        if let Ok(identifier) = self.peek_token()?.clone().into_identifier() {
            // advance identifier
            self.next_token();
            self.expect(Token::Operator(Operator::Assign))?;
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
    use pretty_assertions::assert_eq;

    use crate::parser::expr::Atom;

    use super::*;

    #[test]
    fn var_stmt() {
        assert_eq!(
            Parser::new(vec![
                Token::Var,
                Token::Identifier(String::from("variable")),
                Token::Operator(Operator::Assign),
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

    // parse_var_stmt should fail if expression is not finished with semicolon.
    #[test]
    fn require_semicolon() {
        let mut parser = Parser::new(vec![
            Token::Var,
            Token::Identifier(String::from("foo")),
            Token::Operator(Operator::Assign),
            Token::Number(2.0),
        ]);

        assert_eq!(
            parser.parse_var_stmt().unwrap_err().to_string().as_str(),
            "Expected Semicolon but got unexpected end of input"
        );

        let mut parser = Parser::new(vec![
            Token::Var,
            Token::Identifier(String::from("foo")),
            Token::Operator(Operator::Assign),
            Token::Number(2.0),
            Token::Semicolon,
        ]);

        assert_eq!(
            parser.parse_var_stmt().unwrap(),
            VarStmt {
                identifier: String::from("foo"),
                expr: Expr::Atom(Atom::Number(2.0)),
            }
        )
    }
}
