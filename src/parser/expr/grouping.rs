use anyhow::Result;

use crate::parser::{expr::Expr, Parser, Token};

#[derive(Debug, Clone, PartialEq)]
pub struct Grouping {
    pub expr: Box<Expr>,
}

impl Into<Expr> for Grouping {
    fn into(self) -> Expr {
        Expr::Grouping(self)
    }
}

impl Parser {
    pub fn parse_grouping(&mut self) -> Result<Grouping> {
        self.expect(Token::OpenParenthesis)?;
        let expr = self.parse_expr()?;
        self.expect(Token::CloseParenthesis)?;

        Ok(Grouping {
            expr: Box::new(expr),
        })
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::parser::expr::Atom;

    use super::*;

    #[test]
    fn grouping_expr() {
        let mut parser = Parser::new(vec![
            Token::OpenParenthesis,
            Token::Number(10.0),
            Token::CloseParenthesis,
        ]);

        assert_eq!(
            parser.parse_grouping().unwrap(),
            Grouping {
                expr: Box::new(Expr::Atom(Atom::Number(10.0)))
            }
        )
    }
}
