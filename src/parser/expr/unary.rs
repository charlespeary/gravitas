use std::convert::TryInto;

use anyhow::Result;

use crate::parser::{
    expr::{Expr, Operator},
    Affix, Parser,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Unary {
    pub expr: Box<Expr>,
    pub operator: Operator,
}

impl Into<Expr> for Unary {
    fn into(self) -> Expr {
        Expr::Unary(self)
    }
}

impl Parser {
    pub fn parse_unary(&mut self) -> Result<Unary> {
        let token = self.next_token();
        let bp = token.bp(Affix::Prefix);
        let operator: Operator = token.try_into()?;
        let expr = self.parse_expr_bp(bp)?;

        Ok(Unary {
            expr: Box::new(expr),
            operator,
        })
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::{assert_eq, assert_ne};

    use crate::parser::{
        expr::{Atom, Expr, Grouping},
        Token,
    };

    use super::*;

    #[test]
    fn unary_expr() {
        let mut parser = Parser::new(vec![
            Token::Minus,
            Token::OpenParenthesis,
            Token::Number(10.0),
            Token::CloseParenthesis,
        ]);

        assert_eq!(
            parser.parse_unary().unwrap(),
            Unary {
                operator: Operator::Minus,
                expr: Box::new(Expr::Grouping(Grouping {
                    expr: Box::new(Expr::Atom(Atom::Number(10.0)))
                })),
            }
        );
    }
}
