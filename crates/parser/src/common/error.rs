use codespan_reporting::diagnostic::{Diagnostic, Label};
use logos::Span;

#[derive(Debug)]
pub(crate) struct ParseError {
    pub(crate) span: Span,
    pub(crate) cause: ParseErrorCause,
}

#[derive(Debug, PartialEq)]
pub(crate) enum ParseErrorCause {
    EndOfInput,
}

impl ParseError {
    pub(crate) fn report(&self) -> Diagnostic<()> {
        use ParseErrorCause::*;

        match self.cause {
            EndOfInput => Diagnostic::error().with_message("unexpected end of input"),
        }
    }
}
