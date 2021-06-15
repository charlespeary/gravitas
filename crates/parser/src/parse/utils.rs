use crate::{
    parse::{
        expr::Expr,
        stmt::{Stmt, StmtKind},
        ParseResult, Parser,
    },
    token::Token,
    utils::combine,
};

pub(crate) enum ExprOrStmt {
    Expr(Expr),
    Stmt(Stmt),
}

impl<'t> Parser<'t> {
    pub(super) fn parse_expr_or_stmt(&mut self) -> ParseResult<ExprOrStmt> {
        if self.peek().is_stmt() {
            return Ok(ExprOrStmt::Stmt(self.parse_stmt()?));
        }

        let expr = self.parse_expression()?;

        if self.peek() == Token::Semicolon {
            let semicolon = self.expect(Token::Semicolon)?.span();
            let span = combine(&expr.span, &semicolon);
            Ok(ExprOrStmt::Stmt(Stmt::boxed(
                StmtKind::Expression { expr },
                span,
            )))
        } else {
            Ok(ExprOrStmt::Expr(expr))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        parse::{
            expr::{atom::AtomicValue, ExprKind},
            operator::BinaryOperator,
            Node,
        },
        utils::test::parser::DUMMY_SPAN,
    };

    use super::*;

    fn parse(input: &str) -> ExprOrStmt {
        let mut parser = Parser::new(input);
        parser.parse_expr_or_stmt().unwrap()
    }

    #[test]
    fn parse_expr_or_stmt() {
        if let ExprOrStmt::Expr(expr) = parse("2+3") {
            assert_eq!(
                expr,
                Expr::boxed(
                    ExprKind::Binary {
                        lhs: Expr::boxed(ExprKind::Atom(AtomicValue::Number(2.0)), DUMMY_SPAN),
                        op: Node::new(BinaryOperator::Addition, DUMMY_SPAN),
                        rhs: Expr::boxed(ExprKind::Atom(AtomicValue::Number(3.0)), DUMMY_SPAN)
                    },
                    DUMMY_SPAN
                )
            )
        } else {
            panic!("Expected expression!");
        }

        if let ExprOrStmt::Stmt(stmt) = parse("3+2;") {
            assert_eq!(
                stmt,
                Stmt::boxed(
                    StmtKind::Expression {
                        expr: Expr::boxed(
                            ExprKind::Binary {
                                lhs: Expr::boxed(
                                    ExprKind::Atom(AtomicValue::Number(3.0)),
                                    DUMMY_SPAN
                                ),
                                op: Node::new(BinaryOperator::Addition, DUMMY_SPAN),
                                rhs: Expr::boxed(
                                    ExprKind::Atom(AtomicValue::Number(2.0)),
                                    DUMMY_SPAN
                                )
                            },
                            DUMMY_SPAN
                        )
                    },
                    DUMMY_SPAN
                )
            )
        } else {
            panic!("Expected statement!");
        }
    }
}
