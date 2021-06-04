use crate::token::Token;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use logos::Span;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expect {
    Identifier,
    Literal,
    Expression,
    Statement,
    Token(Token<'static>),
}

impl fmt::Display for Expect {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let str = match self {
            Expect::Identifier => "identifier".to_owned(),
            Expect::Literal => "literal".to_owned(),
            Expect::Expression => "expression".to_owned(),
            Expect::Statement => "statement".to_owned(),
            Expect::Token(t) => format!("{}", t),
        };

        write!(f, "{}", str)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Forbidden {
    TrailingComma,
}

#[derive(Debug)]
pub struct ParseError {
    pub(crate) span: Span,
    pub(crate) cause: ParseErrorCause,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ParseErrorCause {
    EndOfInput,
    UnexpectedToken,
    Expected(Expect),
    ExpectedOneOf(Vec<Token<'static>>),
    NotAllowed(Forbidden),
    // Lexer
    TooMuchDots,
    InvalidNumber,
}

impl ParseError {
    pub(crate) fn report(&self, file_id: usize) -> Diagnostic<usize> {
        use ParseErrorCause::*;
        let span = self.span.clone();

        match &self.cause {
            EndOfInput => Diagnostic::error().with_message("unexpected end of input"),
            UnexpectedToken { .. } => Diagnostic::error()
                .with_message("Encountered unexpected token")
                .with_labels(vec![
                    Label::primary(file_id, span).with_message("wasn't expected")
                ]),
            Expected(e) => Diagnostic::error()
                .with_message(format!("Expected {}", e))
                .with_labels(vec![
                    Label::primary(file_id, span.end..span.end + 1).with_message("but found")
                ]),
            ExpectedOneOf { .. } => Diagnostic::error(),

            _ => Diagnostic::error(),
        }
    }
}
