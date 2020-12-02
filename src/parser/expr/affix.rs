use crate::parser::{
    expr::{Expr, Operator},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Affix {
    pub expr: Box<Expr>,
    // It might be Affix or postfix
    pub operator: Operator,
}

impl Into<Expr> for Affix {
    fn into(self) -> Expr {
        Expr::Affix(self)
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::parser::{
        expr::{Atom, Expr, Grouping},
        Token,
    };

    use super::*;

    #[test]
    fn unary_expr() {
        let mut parser = Parser::new(vec![
            Token::Operator(Operator::Minus),
            Token::OpenParenthesis,
            Token::Number(10.0),
            Token::CloseParenthesis,
        ]);

        assert_eq!(
            parser.parse_expr().unwrap(),
            Expr::Affix(Affix {
                operator: Operator::Minus,
                expr: Box::new(Expr::Grouping(Grouping {
                    expr: Box::new(Expr::Atom(Atom::Number(10.0)))
                })),
            })
        );
    }
}
