use crate::parse::Span;

pub mod error;

#[cfg(test)]
pub(crate) mod test {
    pub(crate) mod lexer {
        use logos::Logos;

        use crate::token::{operator::Operator, Token};

        pub(crate) fn op<'t>(operator: Operator) -> Token<'t> {
            Token::Operator(operator)
        }

        fn tokens(code: &str) -> Vec<Token> {
            Token::lexer(code).collect()
        }

        pub(crate) fn first_token(code: &str) -> Token {
            tokens(code)[0]
        }

        pub(crate) fn assert_error(code: &str) {
            assert_token(code, Token::Error);
        }

        pub(crate) fn assert_token(code: &str, token: Token) {
            assert_eq!(first_token(code), token);
        }

        pub(crate) fn assert_empty(code: &str) {
            assert_tokens(code, &[]);
        }

        pub(crate) fn assert_tokens(code: &str, tokens_to_compare: &[Token]) {
            assert_eq!(tokens(code), tokens_to_compare);
        }
    }

    pub(crate) mod parser {

        use crate::{
            parse::{expr::Expr, stmt::Stmt, Parser, Span},
            utils::error::ParseErrorCause,
        };
        use common::Symbol;
        use lasso::Key;

        pub(crate) const DUMMY_SPAN: Span = 0..0;

        pub(crate) fn expr(input: &str) -> Expr {
            let mut parser = Parser::new(input);
            parser.parse_expression().unwrap()
        }

        pub(crate) fn stmt(input: &str) -> Stmt {
            let mut parser = Parser::new(input);
            parser.parse_stmt().unwrap()
        }

        pub(crate) fn assert_expr(input: &str, expected: &str) {
            assert_eq!(expr(input).to_string(), expected)
        }

        pub(crate) fn assert_stmt(input: &str, expected: &str) {
            assert_eq!(stmt(input).to_string(), expected)
        }

        pub(crate) fn assert_expr_error(input: &str, expected: ParseErrorCause) {
            let mut parser = Parser::new(input);
            assert_eq!(parser.parse_expression().unwrap_err(), expected)
        }

        pub(crate) fn assert_stmt_error(input: &str, expected: ParseErrorCause) {
            let mut parser = Parser::new(input);
            assert_eq!(parser.parse_stmt().unwrap_err(), expected)
        }

        pub(crate) fn symbol(key: usize) -> Symbol {
            Symbol::try_from_usize(key).unwrap()
        }
    }
}

pub(crate) fn combine(a: &Span, b: &Span) -> Span {
    assert!(a.start <= b.end);

    a.start..b.end
}
