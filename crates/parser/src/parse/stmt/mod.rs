use crate::parse::{expr::Expr, ParseResult, Parser, Span, Token};
use derive_more::Display;

#[derive(Debug, Display, Clone, PartialEq)]
pub(crate) enum Stmt {
    #[display(fmt = "{};", expr)]
    Expression { expr: Expr, span: Span },
}

impl<'t> Parser<'t> {
    pub(crate) fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        match self.peek() {
            _ => self.parse_expression_stmt(),
        }
    }

    pub(super) fn parse_expression_stmt(&mut self) -> ParseResult<Stmt> {
        let expr = self.parse_expression()?;
        let lexeme = self.expect(Token::Semicolon)?;

        Ok(Stmt::Expression {
            expr,
            span: lexeme.span(),
        })
    }
}

#[cfg(test)]
mod test {

    use crate::common::test::parser::assert_stmt;

    #[test]
    fn parses_expression_statement() {
        // atomic expression
        assert_stmt("2;", "2;");
        // simple binary expression
        assert_stmt("2 + 2;", "(+ 2 2);");
        // binary expression
        assert_stmt(
            "2 + 2 * 8 >= 10 + 3 ** 4;",
            "(>= (+ 2 (* 2 8)) (+ 10 (** 3 4)));",
        );
        // unary expression
        assert_stmt("!false;", "(! false);");
    }
}
