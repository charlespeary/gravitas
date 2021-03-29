use lasso::{Rodeo, Spur};

use crate::common::error::ParseErrorCause;
use crate::{
    common::error::ParseError,
    token::{Lexeme, Lexer, Token},
};

pub(crate) mod expr;

pub(crate) struct Parser<'a> {
    lexer: Lexer<'a>,
    errors: Vec<ParseError>,
    interner: Rodeo,
}

pub(crate) struct AST;

pub(crate) type ParserOutput<'e> = Result<AST, &'e [ParseError]>;
pub(crate) type ParseResult<T> = Result<T, ParseErrorCause>;
pub(crate) type VtasStringRef = Spur;

impl<'a> Parser<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            lexer: Lexer::new(input),
            errors: vec![],
            interner: Rodeo::new(),
        }
    }

    fn intern(&mut self, str: &str) -> VtasStringRef {
        self.interner.get_or_intern(str)
    }

    fn peek(&mut self) -> ParseResult<Token> {
        self.lexer
            .peek()
            .map(|l| l.token)
            .ok_or(ParseErrorCause::EndOfInput)
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
mod test {}
