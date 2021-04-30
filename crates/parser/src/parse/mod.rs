use crate::common::error::ParseErrorCause;
use crate::{
    common::error::ParseError,
    token::{Lexeme, Lexer, Token},
};
use std::ops::Range;

pub(crate) mod expr;

pub(crate) struct Parser<'a> {
    lexer: Lexer<'a>,
    errors: Vec<ParseError>,
}

pub(crate) struct AST;

pub(crate) type ParserOutput<'e> = Result<AST, &'e [ParseError]>;
pub(crate) type ParseResult<T> = Result<T, ParseErrorCause>;
// For the time being, string is represented as a range of text positions in the source code
pub(crate) type VtasStringRef = Range<usize>;

impl<'a> Parser<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input),
            errors: vec![],
        }
    }

    fn peek(&mut self) -> Token {
        self.lexer
            .peek_nth(0)
            .map(|l| l.token)
            .unwrap_or(Token::Eof)
    }

    fn advance(&mut self) -> ParseResult<Lexeme> {
        self.lexer.next().ok_or(ParseErrorCause::EndOfInput)
    }

    pub(crate) fn parse(&mut self) {
        self.parse_expression();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parser_unexpected_end_of_input_on_advance() {
        let mut parser = Parser::new("");
        assert_eq!(parser.advance().unwrap_err(), ParseErrorCause::EndOfInput);
    }

    #[test]
    fn parser_eof_token_on_peek() {
        let mut parser = Parser::new("");
        assert_eq!(parser.peek(), Token::Eof);
    }
}
