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

impl ExprOrStmt {
    pub(crate) fn is_stmt(&self) -> bool {
        matches!(self, ExprOrStmt::Stmt(_))
    }

    pub(crate) fn is_expr(&self) -> bool {
        matches!(self, ExprOrStmt::Expr(_))
    }
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
    use super::*;

    fn parse(input: &str) -> ExprOrStmt {
        let mut parser = Parser::new(input);
        parser.parse_expr_or_stmt().unwrap()
    }

    #[test]
    fn parse_expr_or_stmt() {
        assert!(parse("2+2").is_expr());
        assert!(parse("2+2;").is_stmt());
    }
}
