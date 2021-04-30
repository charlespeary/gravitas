use crate::{
    common::error::ParseErrorCause,
    parse::{expr::atom::Atom, ParseResult, Parser},
    token::{operator::Operator, Token},
};

pub(crate) mod atom;

#[derive(Debug, Clone)]
pub(crate) enum SExpr {
    Atom(Atom),
    Cons(Operator, Vec<SExpr>),
}

impl<'a> Parser<'a> {
    pub(super) fn parse_expression(&mut self) -> ParseResult<SExpr> {
        self.parse_expression_bp(0)
        // Ok(match self.peek()? {
        //     _ => try_expr!(self.parse_atom()),
        // })
    }

    pub(super) fn parse_expression_bp(&mut self, min_bp: u8) -> ParseResult<SExpr> {
        let mut lhs: SExpr = self.parse_atom()?.into();

        loop {
            let operator = match self.peek() {
                Token::Operator(operator) => operator,
                Token::Eof => break,
                _ => return Err(ParseErrorCause::UnexpectedToken),
            };

            dbg!(operator);

            let (l_bp, r_bp) = operator.bp();

            if l_bp < min_bp {
                break;
            }

            // Advance the operator
            dbg!(self.peek());
            self.advance()?;
            dbg!(self.peek());
            let rhs = self.parse_expression_bp(r_bp)?;
            lhs = SExpr::Cons(operator, vec![lhs, rhs]);
        }

        Ok(lhs)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn sexpr(input: &str) -> SExpr {
        let mut parser = Parser::new(input);
        parser.parse_expression().unwrap()
    }

    #[test]
    fn parser_parses_simple_expressions() {
        dbg!(sexpr("\"foo\" + 2"));
        // assert_eq!(sexpr("2 + 2"), SExpr::Cons(Operator::Plus, vec![]))
    }
}
