use crate::{
    parse::{Node, ParseResult, Parser, Symbol},
    token::{
        constants::{CLOSE_PARENTHESIS, OPEN_PARENTHESIS},
        Token,
    },
    utils::{
        combine,
        error::{Forbidden, ParseErrorCause},
    },
};

pub(crate) type Param = Node<Symbol>;
// (a, b, c)
pub(crate) type Params = Node<Vec<Param>>;

impl<'t> Parser<'t> {
    pub(super) fn parse_params(&mut self) -> ParseResult<Params> {
        let (open_parenthesis, closing_token) = {
            // we encountered closure opening so we will have to expect closing bar
            // after the end of the parameters list
            let (opening_token, closing_token) = if self.peek() == Token::Bar {
                (Token::Bar, Token::Bar)
            } else {
                (OPEN_PARENTHESIS, CLOSE_PARENTHESIS)
            };

            (self.expect(opening_token)?.span(), closing_token)
        };

        let mut args: Vec<Param> = Vec::new();

        loop {
            let next = self.peek();
            if next == closing_token || !next.is_identifier() {
                break;
            }

            let (identifier, arg_lexeme) = self.expect_identifier()?;
            let arg = Param::new(identifier, arg_lexeme.span());
            args.push(arg);

            if self.peek() != closing_token {
                self.expect(Token::Comma)?;

                if !self.peek().is_identifier() {
                    return Err(ParseErrorCause::NotAllowed(Forbidden::TrailingComma));
                }
            }
        }

        let close_parenthesis = self.expect(closing_token)?.span();

        Ok(Params::new(
            args,
            combine(&open_parenthesis, &close_parenthesis),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::test::parser::symbol;

    fn assert_args(input: &str, args: Params) {
        let mut parser = Parser::new(input);
        assert_eq!(parser.parse_params().unwrap(), args);
    }

    #[test]
    fn parser_parses_arguments() {
        assert_args("()", Params::new(vec![], 0..2));

        assert_args("(a)", Params::new(vec![Param::new(symbol(0), 2..3)], 0..3));
        assert_args(
            "(a, b)",
            Params::new(
                vec![Param::new(symbol(0), 2..3), Param::new(symbol(1), 4..5)],
                0..6,
            ),
        );
        assert_args(
            "(a, b, c)",
            Params::new(
                vec![
                    Param::new(symbol(0), 2..3),
                    Param::new(symbol(1), 4..5),
                    Param::new(symbol(2), 6..7),
                ],
                0..8,
            ),
        );
    }

    #[test]
    fn parser_doesnt_allow_trailing_comma_while_parsing_args() {
        let mut parser = Parser::new("(a,)");
        assert_eq!(
            parser.parse_params().unwrap_err(),
            ParseErrorCause::NotAllowed(Forbidden::TrailingComma)
        );
    }
}
