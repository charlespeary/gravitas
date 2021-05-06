use crate::{
    common::combine,
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

    pub(super) fn parse_if_expr(&mut self) -> ExprResult {
        let start_span = self.expect(Token::If)?.span();
        let expr = self.parse_expression()?;
        let body = self.parse_block_expr()?;
        let else_expr = if self.peek() == Token::Else {
            let else_keyword = self.advance()?;
            let else_body = if self.peek() == Token::If {
                self.parse_if_expr()?
            } else {
                self.parse_block_expr()?
            };

            Some(else_body)
        } else {
            None
        };

        let end_span = else_expr
            .as_ref()
            .map(|expr| &expr.span)
            .unwrap_or(&body.span);

        let span = combine(&start_span, &end_span);

        Ok(Expr::new(
            ExprKind::If {
                expr,
                else_expr,
                body,
            },
            span,
        ))
    }
}

#[cfg(test)]
mod test {
    use crate::{
        common::{
            error::{Expect, ParseErrorCause},
            test::parser::{assert_expr, assert_expr_error},
        },
        token::constants::{CLOSE_BRACKET, OPEN_BRACKET},
    };

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

        assert_expr_error("{", ParseErrorCause::Expected(Expect::Token(CLOSE_BRACKET)))
    }

    #[test]
    fn parser_parses_if_expressions() {
        assert_expr("if true {  }", "if true {  }");
        assert_expr("if true {  } else {  }", "if true {  } else {  }");
        assert_expr("if true { 5 } else { 10 }", "if true { 5 } else { 10 }");

        assert_expr_error("if", ParseErrorCause::Expected(Expect::Expression));
        assert_expr_error(
            "if true",
            ParseErrorCause::Expected(Expect::Token(OPEN_BRACKET)),
        );

        assert_expr(
            "if true { 10 } else if false { 5 }",
            "if true { 10 } else if false { 5 }",
        );

        assert_expr(
            "if 2 == 3 { 1 } else if 3 == 4 { 0 } else { 2 }",
            "if (== 2 3) { 1 } else if (== 3 4) { 0 } else { 2 }",
        );
    }
}
