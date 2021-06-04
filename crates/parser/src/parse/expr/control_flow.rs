use crate::parse::utils::ExprOrStmt;
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
    utils::combine,
};

impl<'t> Parser<'t> {
    pub(super) fn parse_block_expr(&mut self) -> ExprResult {
        let open_bracket = self.expect(OPEN_BRACKET)?.span();
        let mut stmts: Vec<Stmt> = vec![];
        let mut return_expr = None;
        loop {
            let next = self.peek();
            if next == CLOSE_BRACKET || next == Token::Eof {
                break;
            }
            match self.parse_expr_or_stmt()? {
                ExprOrStmt::Expr(expr) => {
                    return_expr = Some(expr);
                }
                ExprOrStmt::Stmt(stmt) => {
                    stmts.push(stmt);
                }
            }
        }

        let close_bracket = self.expect(CLOSE_BRACKET)?.span();
        let span = combine(&open_bracket, &close_bracket);
        Ok(Expr::boxed(ExprKind::Block { return_expr, stmts }, span))
    }

    pub(super) fn parse_if_expr(&mut self) -> ExprResult {
        let start_span = self.expect(Token::If)?.span();
        let condition = self.parse_expression()?;
        let body = self.parse_block_expr()?;
        let else_expr = if self.peek() == Token::Else {
            self.advance()?;
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

        Ok(Expr::boxed(
            ExprKind::If {
                condition,
                else_expr,
                body,
            },
            span,
        ))
    }

    pub(super) fn parse_while_expr(&mut self) -> ExprResult {
        let keyword = self.expect(Token::While)?.span();
        let condition = self.parse_expression()?;
        let body = self.parse_block_expr()?;
        let span = combine(&keyword, &body.span);

        Ok(Expr::boxed(ExprKind::While { condition, body }, span))
    }

    pub(super) fn parse_break_expr(&mut self) -> ExprResult {
        let keyword = self.expect(Token::Break)?.span();
        let return_expr = if self.peek().is_expr() {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let span = if let Some(return_expr) = &return_expr {
            combine(&keyword, &return_expr.span)
        } else {
            keyword
        };

        Ok(Expr::boxed(ExprKind::Break { return_expr }, span))
    }

    pub(super) fn parse_continue_expr(&mut self) -> ExprResult {
        let keyword = self.expect(Token::Continue)?.span();

        Ok(Expr::boxed(ExprKind::Continue, keyword))
    }
}

#[cfg(test)]
mod test {
    use crate::{
        token::constants::{CLOSE_BRACKET, OPEN_BRACKET},
        utils::{
            error::{Expect, ParseErrorCause},
            test::parser::{assert_expr, assert_expr_error},
        },
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

    #[test]
    fn parser_parses_while_expressions() {
        assert_expr("while true { }", "while true {  }");
        assert_expr("while 3 < 10 { 5 }", "while (< 3 10) { 5 }");
        assert_expr(
            "while x < 10 { let x = x + 1; }",
            "while (< $symbol 10) { let $symbol = (+ $symbol 1); }",
        );
        assert_expr(
            "while foo < 20 { foo = foo + 1; }",
            "while (< $symbol 20) { $symbol = (+ $symbol 1); }",
        );
    }

    #[test]
    fn parser_parses_break_expressions() {
        assert_expr("break", "break");
        assert_expr("break 5", "break 5");
        assert_expr("break foo + 10", "break (+ $symbol 10)");
        assert_expr("break foo <= bar", "break (<= $symbol $symbol)");
    }

    #[test]
    fn parser_parses_continue_expressions() {
        assert_expr("continue", "continue");
    }
}
