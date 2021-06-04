use crate::{
    parse::{expr::Expr, stmt::Stmt},
    token::{constants::IDENTIFIER, Lexeme, Lexer, Token},
    utils::error::{Expect, ParseError, ParseErrorCause},
};
use lasso::{Rodeo, Spur};
use std::{fmt, mem::discriminant, ops::Range};

pub(crate) mod expr;
pub(crate) mod operator;
pub(crate) mod pieces;
pub(crate) mod stmt;
pub(crate) mod utils;

pub(crate) struct Parser<'t> {
    lexer: Lexer<'t>,
    symbols: Rodeo,
}

#[derive(Debug)]
pub struct Program {
    pub ast: Vec<Stmt>,
    pub symbols: Rodeo,
}

pub(crate) type ParserOutput = Result<Program, Vec<ParseError>>;
pub(crate) type ParseResult<'t, T> = Result<T, ParseErrorCause>;
pub(crate) type ExprResult<'t> = ParseResult<'t, Expr>;
pub(crate) type StmtResult<'t> = ParseResult<'t, Stmt>;

pub type Number = f64;
pub type Symbol = Spur;
pub type Span = Range<usize>;

#[derive(Debug, Clone)]
pub struct Node<T> {
    pub kind: T,
    pub span: Span,
}

impl<T> fmt::Display for Node<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)?;
        Ok(())
    }
}

impl<T> Node<T> {
    pub(crate) fn new(kind: T, span: Span) -> Self {
        Self { kind, span }
    }
}

impl<T> Node<Box<T>> {
    pub(crate) fn boxed(kind: T, span: Span) -> Self {
        Self {
            kind: Box::new(kind),
            span,
        }
    }
}

impl<T> PartialEq for Node<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl<'t> Parser<'t> {
    pub(crate) fn new(input: &'t str) -> Self {
        Self {
            lexer: Lexer::new(input),
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

        Err(ParseErrorCause::Expected(Expect::Token(expected)))
    }

    fn expect_identifier(&mut self) -> ParseResult<(Symbol, Lexeme)> {
        if let Ok(next) = self.advance() {
            if discriminant(&next.token) == discriminant(&IDENTIFIER) {
                return Ok((next.intern_key.unwrap(), next));
            }
        }

        Err(ParseErrorCause::Expected(Expect::Identifier))
    }

    pub(crate) fn parse(mut self) -> ParserOutput {
        let mut ast = Vec::new();
        let mut errors = Vec::new();

        while self.peek() != Token::Eof {
            match self.parse_stmt() {
                Ok(stmt) => {
                    ast.push(stmt);
                }
                Err(cause) => {
                    let parse_error = ParseError {
                        cause,
                        span: self.lexer.current_span(),
                    };
                    errors.push(parse_error);

                    // discard every expression until we encounter a new statement
                    loop {
                        let next = self.peek();
                        if next.is_stmt() || next == Token::Eof {
                            break;
                        }
                        self.advance().unwrap();
                    }
                }
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(Program {
                ast,
                symbols: self.symbols,
            })
        }
    }

    fn construct_node<T>(&mut self, val: T) -> ParseResult<Node<T>> {
        let lexeme = self.advance()?;
        Ok(Node {
            kind: val,
            span: lexeme.span(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parse::expr::atom::AtomicValue;

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

    #[test]
    fn parser_expects_token() {
        let mut parser = Parser::new("class fn");
        // it advances and returns the advanced lexeme
        assert_eq!(
            parser.expect(Token::Class).unwrap(),
            Lexeme {
                token: Token::Class,
                slice: "class",
                span_start: 0,
                span_end: 5,
                intern_key: None
            }
        );
        // it reports an error if there isn't what we expect
        assert_eq!(
            parser.expect(Token::Class).unwrap_err(),
            ParseErrorCause::Expected(Expect::Token(Token::Class))
        );
    }

    #[test]
    fn parser_expects_identifiers() {
        let mut parser = Parser::new("foo fn");
        let (identifier, _) = parser.expect_identifier().unwrap();
        assert_eq!(parser.symbols.resolve(&identifier), "foo");
        assert_eq!(
            parser.expect_identifier().unwrap_err(),
            ParseErrorCause::Expected(Expect::Identifier)
        );
    }

    #[test]
    fn parser_constructs_spanned() {
        let mut parser = Parser::new("2");
        let two = AtomicValue::Number(2.0);

        assert_eq!(
            parser.construct_node(two.clone()).unwrap(),
            Node {
                kind: two,
                span: 0..1
            }
        )
    }
}
