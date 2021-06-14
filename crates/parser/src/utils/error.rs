use crate::token::Token;
use codespan_reporting::diagnostic::{Diagnostic, Label};
use common::{CompilerDiagnostic, Symbol, Symbols};
use logos::Span;
use std::fmt::{self, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum Expect {
    Identifier,
    Literal,
    Expression,
    Token(Token<'static>),
}

impl fmt::Display for Expect {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let str = match self {
            Expect::Identifier => "identifier".to_owned(),
            Expect::Literal => "literal".to_owned(),
            Expect::Expression => "expression".to_owned(),
            Expect::Token(t) => format!("{}", t),
        };

        write!(f, "{}", str)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Forbidden {
    TrailingComma,
}

#[derive(Debug)]
pub struct ParseError {
    pub span: Span,
    pub cause: ParseErrorCause,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorCause {
    EndOfInput,
    UnexpectedToken,
    Expected(Expect),
    NotAllowed(Forbidden),
    UsedBeforeInitialization,
    UsedOutsideLoop,
    UsedOutsideClass,
    CantInheritFromItself,
    SuperclassDoesntExist,
}

impl CompilerDiagnostic for ParseError {
    fn report(&self, file_id: usize, symbols: &Symbols) -> Diagnostic<usize> {
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
            UsedBeforeInitialization => Diagnostic::error()
                .with_message("Variable was used before initialization")
                .with_labels(vec![Label::primary(file_id, span)]),
            UsedOutsideLoop => Diagnostic::error()
                .with_message("Break or continue must be used inside loops")
                .with_labels(vec![
                    Label::primary(file_id, span).with_message("...but was used here")
                ]),
            CantInheritFromItself => Diagnostic::error()
                .with_message("Class can't inherit from itself")
                .with_labels(vec![
                    Label::primary(file_id, span).with_message("...but it did here")
                ]),
            SuperclassDoesntExist => Diagnostic::error()
                .with_message("Tried to inherit from a superclass that doesn't exist")
                .with_labels(vec![Label::primary(file_id, span)]),
            UsedOutsideClass => Diagnostic::error()
                .with_message("Use of 'super' || 'this' is forbidden outside class methods")
                .with_labels(vec![Label::primary(file_id, span)]),
            _ => Diagnostic::error().with_message("TODO"),
        }
    }
}
