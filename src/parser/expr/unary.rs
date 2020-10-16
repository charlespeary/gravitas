use anyhow::Result;

use crate::parser::{expr::{Affix, Expr, Operator}, Parser};

pub(crate) struct Unary {
    expr: Box<Expr>,
    operator: Operator,
}

impl Into<Expr> for Unary {
    fn into(self) -> Expr {
        Expr::Unary(self)
    }
}

impl Parser {
    fn parse_unary(&mut self) -> Result<Unary> {
        let operator: Operator = self.next_token().into();
        let bp = operator.bp(Affix::Prefix);
        let expr = self.parse_expr_bp(bp)?;

        Ok(Unary {
            expr: Box::new(expr),
            operator,
        })
    }
}
