use crate::common::combine;
use crate::{
    parse::{
        expr::{Expr, ExprKind},
        stmt::Stmt,
        ExprResult, Parser,
    },
    token::{
        constants::{CLOSE_BRACKET, OPEN_BRACKET},
        Token,
    },
};

impl<'t> Parser<'t> {
    pub(super) fn parse_block_expr(&mut self) -> ExprResult {
        let open_bracket = self.expect(OPEN_BRACKET)?.span();
        let mut stmts: Vec<Stmt> = vec![];
        loop {
            let next = self.peek();
            if next == CLOSE_BRACKET || next == Token::Eof {
                break;
            }

            if next.is_stmt() {
                stmts.push(self.parse_stmt()?);
            } else {
                break;
            }
        }
        let return_expr = if self.peek().is_expr() {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let close_bracket = self.expect(CLOSE_BRACKET)?.span();
        let span = combine(&open_bracket, &close_bracket);
        Ok(Expr::new(ExprKind::Block { return_expr, stmts }, span))
    }
}

#[cfg(test)]
mod test {
    use crate::common::test::parser::assert_expr;

    #[test]
    fn parser_parses_block_expressions() {
        assert_expr("{  }", "{  }");
        assert_expr("{ 2 }", "{ 2 }");
        assert_expr("{ let x = 10; }", "{ let $symbol = 10; }");
        assert_expr("{ let x = 10; 5 }", "{ let $symbol = 10; 5 }");
        assert_expr(
            "{ let x = 10; let y = 5; 5 }",
            "{ let $symbol = 10; let $symbol = 5; 5 }",
        );
    }
}
