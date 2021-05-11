use crate::{
    common::combine,
    parse::{expr::Expr, Node, Parser, Span, StmtResult, Symbol},
    token::{operator::Operator, Token},
};
use derive_more::Display;

pub(crate) type Stmt = Node<Box<StmtKind>>;

pub(crate) mod fun;

#[derive(Debug, Display, Clone, PartialEq)]
pub(crate) enum StmtKind {
    #[display(fmt = "{};", expr)]
    Expression { expr: Expr },
    #[display(fmt = "let $symbol = {};", expr)]
    VariableDeclaration { identifier: Symbol, expr: Expr },
}

impl<'t> Parser<'t> {
    pub(crate) fn parse_stmt(&mut self) -> StmtResult {
        match self.peek() {
            Token::Let => self.parse_variable_declaration(),
            _ => self.parse_expression_stmt(),
        }
    }

    pub(super) fn parse_expression_stmt(&mut self) -> StmtResult {
        let expr = self.parse_expression()?;
        let semicolon = self.expect(Token::Semicolon)?.span();
        let span = combine(&expr.span, &semicolon);

        Ok(Stmt::boxed(StmtKind::Expression { expr }, span))
    }

    pub(super) fn parse_variable_declaration(&mut self) -> StmtResult {
        let let_keyword = {
            let lexeme = self.expect(Token::Let)?;
            lexeme.span()
        };
        let (identifier, _) = self.expect_identifier()?;
        self.expect(Token::Operator(Operator::Assign))?;
        let expr = self.parse_expression()?;
        let semicolon = self.expect(Token::Semicolon)?;
        let span = combine(&let_keyword, &semicolon.span());
        Ok(Stmt::boxed(
            StmtKind::VariableDeclaration { identifier, expr },
            span,
        ))
    }
}

#[cfg(test)]
mod test {

    use crate::{
        common::{
            error::{Expect, ParseErrorCause},
            test::parser::{assert_stmt, assert_stmt_error},
        },
        token::Token,
    };

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

    #[test]
    fn expression_statement_should_expect_semicolon() {
        fn assert_semicolon(input: &str) {
            assert_stmt_error(
                input,
                ParseErrorCause::Expected(Expect::Token(Token::Semicolon)),
            );
        }
        assert_semicolon("2");
        assert_semicolon("2 + 2");
        assert_semicolon("2 + 2 >= 10");
    }

    #[test]
    fn parses_variable_declaration() {
        assert_stmt("let foo = 10;", "let $symbol = 10;");
        assert_stmt("let bar = 2 + 2 >= 10;", "let $symbol = (>= (+ 2 2) 10);");
    }
}
