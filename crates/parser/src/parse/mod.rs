use crate::token::operator::Operator;
use crate::{
    common::error::{ParseError, ParseErrorCause},
    token::{
        constants::{IDENTIFIER, OPERATOR},
        Lexeme, Lexer, Token,
    },
};
use derive_more::Display;
use lasso::{Rodeo, Spur};
use std::mem::discriminant;
use std::ops::Range;

pub(crate) mod expr;
pub(crate) mod operator;
pub(crate) mod stmt;

pub(crate) struct Parser<'t> {
    lexer: Lexer<'t>,
    errors: Vec<ParseError>,
    symbols: Rodeo,
}

pub struct AST;

pub(crate) type ParserOutput<'t> = Result<AST, &'t [ParseError]>;
pub(crate) type ParseResult<'t, T> = Result<T, ParseErrorCause>;
pub type Number = f64;
pub type Symbol = Spur;
pub type Span = Range<usize>;

#[derive(Debug, Clone, Display)]
#[display(fmt = "{}", val)]
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

impl<'t> Parser<'t> {
    pub(crate) fn new(input: &'t str) -> Self {
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

    fn expect(&mut self, expected: Token<'static>) -> ParseResult<Lexeme> {
        if let Ok(next) = self.advance() {
            if next.token == expected {
                return Ok(next);
            }
        }

        Err(ParseErrorCause::Expected(expected))
    }

    fn expect_identifier(&mut self) -> ParseResult<Symbol> {
        if let Ok(next) = self.advance() {
            if discriminant(&next.token) == discriminant(&IDENTIFIER) {
                return Ok(next.intern_key.unwrap());
            }
        }

        Err(ParseErrorCause::ExpectedIdentifier)
    }

    pub(crate) fn parse(&mut self) {
        self.parse_expression();
    }

    fn construct_spanned<T>(&mut self, val: T) -> ParseResult<Spanned<T>> {
        let lexeme = self.advance()?;
        Ok(Spanned {
            val,
            span: lexeme.span(),
        })
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
