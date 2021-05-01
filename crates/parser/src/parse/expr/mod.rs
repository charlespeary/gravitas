use crate::{
    common::error::ParseErrorCause,
    parse::{expr::atom::Atom, operator::BinaryOperator, ParseResult, Parser, Spanned},
    token::Token,
};
use std::convert::TryInto;

pub(crate) mod atom;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Binary {
    pub(crate) lhs: Box<Expr>,
    pub(crate) op: Spanned<BinaryOperator>,
    pub(crate) rhs: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expr {
    Atom(Atom),
    Binary(Binary),
}

// Macro to implement From traits for all the AST little expression pieces
macro_rules! impl_from_for_ast_pieces {
    ( $( $ast_piece: ident), *) => {
        $(
            impl From<$ast_piece> for Expr {
                fn from(val: $ast_piece) -> Self { Expr::$ast_piece(val)}
            }
        )*
    }
}

impl_from_for_ast_pieces!(Atom, Binary);

impl<'a> Parser<'a> {
    pub(super) fn parse_expression(&mut self) -> ParseResult<Expr> {
        self.parse_expression_bp(0)
    }

    pub(super) fn parse_expression_bp(&mut self, min_bp: u8) -> ParseResult<Expr> {
        let mut lhs: Expr = self.parse_atom()?.into();

        loop {
            let operator = match self.peek() {
                Token::Operator(operator) => operator,
                Token::Eof => break,
                _ => return Err(ParseErrorCause::UnexpectedToken),
            };

            let (l_bp, r_bp) = operator.bp();

            if l_bp < min_bp {
                break;
            }

            // Advance and construct spanned operator
            let op = {
                let lexeme = self.advance()?;
                Spanned {
                    val: operator.try_into()?,
                    span: lexeme.span(),
                }
            };
            let rhs = self.parse_expression_bp(r_bp)?;
            lhs = Binary {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            }
            .into();
        }

        Ok(lhs)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck_macros::quickcheck;

    use crate::parse::{
        expr::atom::{test::num, AtomicValue},
        operator::BinaryOperator,
        Number,
    };

    fn expr(input: &str) -> Expr {
        let mut parser = Parser::new(input);
        parser.parse_expression().unwrap()
    }

    fn spanned<T>(val: T) -> Spanned<T>
    where
        T: PartialEq,
    {
        Spanned { val, span: 0..0 }
    }

    #[quickcheck]
    fn q_parser_parses_simple_numeric_binary_expressions(a: Number, b: Number, op: BinaryOperator) {
        // TODO: make it work by excluding spans from the equality check
        assert_eq!(
            expr(&format!("{} {} {}", a, op, b)),
            Expr::Binary(Binary {
                lhs: Box::new(num(a)),
                op: spanned(op),
                rhs: Box::new(num(b))
            })
        )
    }
}
