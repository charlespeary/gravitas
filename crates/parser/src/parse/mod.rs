use crate::{
    common::error::{ParseError, ParseErrorCause},
    token::{Lexeme, Lexer, Token},
};
use lasso::{Rodeo, Spur};
use std::ops::Range;

pub(crate) mod expr;
pub(crate) mod operator;

pub(crate) struct Parser<'a> {
    lexer: Lexer<'a>,
    errors: Vec<ParseError>,
    symbols: Rodeo,
}

pub struct AST;

pub(crate) type ParserOutput<'e> = Result<AST, &'e [ParseError]>;
pub(crate) type ParseResult<T> = Result<T, ParseErrorCause>;
pub type Number = f64;
// For the time being, string is represented as a range of text positions in the source code
pub type Symbol = Spur;
pub type Span = Range<usize>;

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub val: T,
    pub span: Span,
}

impl<T> PartialEq for Spanned<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val
    }
}

impl<'a> Parser<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input),
            errors: vec![],
            symbols: Rodeo::new(),
        }
    }

    fn peek(&mut self) -> Token {
        self.lexer
            .peek_nth(0)
            .map(|l| l.token)
            .unwrap_or(Token::Eof)
    }

    fn intern(&mut self, string: &str) -> Symbol {
        self.symbols.get_or_intern(string)
    }

    fn advance(&mut self) -> ParseResult<Lexeme> {
        self.lexer
            .next()
            .as_mut()
            .map(|lexeme| {
                let intern_key = match lexeme.token {
                    Token::String(string) => Some(self.intern(string)),
                    Token::Identifier(identifier) => Some(self.intern(identifier)),
                    _ => None,
                };
                lexeme.intern_key = intern_key;
                *lexeme
            })
            .ok_or(ParseErrorCause::EndOfInput)
    }

    pub(crate) fn parse(&mut self) {
        self.parse_expression();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parser_interns_strings() {
        let mut parser = Parser::new("\"string literal\"");
        assert!(parser.advance().unwrap().intern_key.is_some());
    }

    #[test]
    fn parser_interns_identifiers() {
        let mut parser = Parser::new("foo");
        assert!(parser.advance().unwrap().intern_key.is_some());
    }

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
