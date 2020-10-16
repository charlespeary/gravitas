use anyhow::Result;

use crate::parser::{expr::{Affix, Expr}, Parser, Token};

pub(crate) struct Grouping {
    expr: Box<Expr>
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