use anyhow::Result;

use crate::parser::{Expr, Parser, Stmt, Token};

/// Statement used to evaluate the expression.
/// `ExprStmt => Expr;`
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
    /// Parse expression statement.
    /// Returns an error if the expression is invalid or not followed by a semicolon.
    pub fn parse_expr_stmt(&mut self) -> Result<ExprStmt> {
        let expr = self.parse_expr()?;
        self.expect(Token::Semicolon)?;
        Ok(ExprStmt { expr })
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::parser::expr::{Atom, Binary, Expr, Operator};

    use super::*;

    // Parse ExprStmt correctly
    #[test]
    fn parse_expr_stmt() {
        let mut parser = Parser::new(vec![
            Token::Number(2.0),
            Token::Operator(Operator::Plus),
            Token::Number(2.0),
            Token::Semicolon,
        ]);

        assert_eq!(
            parser
                .parse_expr_stmt()
                .expect("Couldn't parse the expr_stmt from given tokens"),
            ExprStmt {
                expr: Expr::Binary(Binary {
                    lhs: Box::new(Expr::Atom(Atom::Number(2.0))),
                    operator: Operator::Plus,
                    rhs: Box::new(Expr::Atom(Atom::Number(2.0))),
                })
            }
        );
    }

    // parse_expr_stmt should fail if expression is not finished with semicolon.
    #[test]
    fn require_semicolon() {
        let mut parser = Parser::new(vec![
            Token::Number(2.0),
            Token::Operator(Operator::Plus),
            Token::Number(2.0),
        ]);

        assert_eq!(
            parser.parse_expr_stmt().unwrap_err().to_string().as_str(),
            "Expected Semicolon but got unexpected end of input"
        )
    }
}
