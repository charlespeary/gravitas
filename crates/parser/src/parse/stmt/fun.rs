use crate::{
    common::{
        combine,
        error::{Forbidden, ParseErrorCause},
    },
    parse::{Node, ParseResult, Parser, Symbol},
    token::{
        constants::{CLOSE_PARENTHESIS, OPEN_PARENTHESIS},
        Token,
    },
};

pub(crate) type Arg = Node<Symbol>;
// (a, b, c)
pub(crate) type Args = Node<Vec<Arg>>;

impl<'t> Parser<'t> {
    pub(super) fn parse_args(&mut self) -> ParseResult<Args> {
        let open_parenthesis = self.expect(OPEN_PARENTHESIS)?.span();

        let mut args: Vec<Arg> = Vec::new();

        loop {
            let next = self.peek();
            if next == CLOSE_PARENTHESIS || !next.is_identifier() {
                break;
            }

            let (identifier, arg_lexeme) = self.expect_identifier()?;
            let arg = Arg::new(identifier, arg_lexeme.span());
            args.push(arg);

            if self.peek() != CLOSE_PARENTHESIS {
                self.expect(Token::Comma)?;

                if !self.peek().is_identifier() {
                    return Err(ParseErrorCause::NotAllowed(Forbidden::TrailingComma));
                }
            }
        }

        let close_parenthesis = self.expect(CLOSE_PARENTHESIS)?.span();

        Ok(Args::new(
            args,
            combine(&open_parenthesis, &close_parenthesis),
        ))
    }
}

#[cfg(test)]
mod test {
    use crate::{
        common::{
            error::{Forbidden, ParseErrorCause},
            test::parser::symbol,
        },
        parse::{
            stmt::fun::{Arg, Args},
            Parser, Symbol,
        },
    };

    fn assert_args(input: &str, args: Args) {
        let mut parser = Parser::new(input);
        assert_eq!(parser.parse_args().unwrap(), args);
    }

    #[test]
    fn parser_parses_arguments() {
        assert_args("()", Args::new(vec![], 0..2));

        assert_args("(a)", Args::new(vec![Arg::new(symbol(0), 2..3)], 0..3));
        assert_args(
            "(a, b)",
            Args::new(
                vec![Arg::new(symbol(0), 2..3), Arg::new(symbol(1), 4..5)],
                0..6,
            ),
        );
        assert_args(
            "(a, b, c)",
            Args::new(
                vec![
                    Arg::new(symbol(0), 2..3),
                    Arg::new(symbol(1), 4..5),
                    Arg::new(symbol(2), 6..7),
                ],
                0..8,
            ),
        );
    }

    #[test]
    fn parser_doesnt_allow_trailing_comma_while_parsing_args() {
        let mut parser = Parser::new("(a,)");
        assert_eq!(
            parser.parse_args().unwrap_err(),
            ParseErrorCause::NotAllowed(Forbidden::TrailingComma)
        );
    }
}
