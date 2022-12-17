use crate::{
    parse::{
        expr::{Expr, ExprKind},
        ExprResult, Node, Parser,
    },
    token::{
        constants::{ASSIGN, DOT},
        Token,
    },
    utils::combine,
};
use common::{Number, ProgramText};
use std::fmt;

pub type VariableProperty = Node<String>;

#[derive(Debug, Clone, PartialEq)]
pub enum AtomicValue {
    Boolean(bool),
    Number(Number),
    Text(ProgramText),
    Identifier { name: String, is_assignment: bool },
}

impl fmt::Display for AtomicValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AtomicValue::*;

        match self {
            Boolean(val) => {
                write!(f, "{}", val)?;
            }
            Number(val) => {
                write!(f, "{}", val)?;
            }
            Text(text) => {
                write!(f, "{}", text)?;
            }
            Identifier { name, .. } => {
                write!(f, "{}", name)?;
            }
        }

        Ok(())
    }
}

impl<'t> Parser<'t> {
    pub(super) fn parse_atom_expr(&mut self) -> ExprResult {
        let lexeme = self.advance()?;
        let atom_span = lexeme.span();

        let val = match lexeme.token {
            Token::Bool(val) => AtomicValue::Boolean(val),
            Token::Number(val) => AtomicValue::Number(val),
            // It's safe to unwrap because these strings should be interned during advance()
            // If it panics then we have a bug in our code
            Token::String(str) => AtomicValue::Text(str.to_owned()),
            Token::Identifier(identifier) => {
                let name = identifier.to_owned();
                let is_assignment = self.peek() == ASSIGN;

                AtomicValue::Identifier {
                    name,
                    is_assignment,
                }
            }
            t => {
                panic!("Encountered {:?} while parsing atom", t);
            }
        };

        Ok(Expr::boxed(ExprKind::Atom(val), atom_span))
    }
}

#[cfg(test)]
pub(crate) mod test {
    use quickcheck_macros::quickcheck;

    use super::*;

    #[test]
    fn parses_atom_booleans() {
        let mut parser = Parser::new("true false");
        assert_eq!(
            parser.parse_atom_expr().unwrap(),
            Expr::boxed(ExprKind::Atom(AtomicValue::Boolean(true)), 0..4)
        );
        assert_eq!(
            parser.parse_atom_expr().unwrap(),
            Expr::boxed(ExprKind::Atom(AtomicValue::Boolean(false)), 5..10)
        )
    }

    #[quickcheck]
    fn parses_atom_numbers(number: f64) {
        if number.is_nan() || number.is_infinite() {
            return;
        }

        let num_as_str = number.to_string();

        let mut parser = Parser::new(&num_as_str);

        assert_eq!(
            parser.parse_atom_expr().unwrap(),
            Expr::boxed(
                ExprKind::Atom(AtomicValue::Number(number)),
                0..num_as_str.len()
            )
        )
    }

    #[quickcheck]
    #[test]
    fn parses_atom_strings(text: String) {
        let text = text.replace("\"", "");
        // Quote the string, so it's lexed as a string token and not an identifier
        // Also, get rid of the quotes because they are not allowed inside our string representation
        // and quickcheck generates those. It'd be a good idea to create our own implementation of that random string.
        let quoted_text = format!("\"{}\"", &text);
        let mut parser = Parser::new(&quoted_text);

        let parsed_string = parser.parse_atom_expr().unwrap();

        assert_eq!(
            parsed_string,
            Expr::boxed(
                ExprKind::Atom(AtomicValue::Text(text.clone())),
                0..text.len() + 2
            )
        );
    }

    #[test]
    fn parses_atom_identifiers() {
        fn test_identifier(identifier: &str) {
            let mut parser = Parser::new(identifier);

            let parsed_identifier = parser.parse_atom_expr().unwrap();
            assert_eq!(
                parsed_identifier,
                Expr::boxed(
                    ExprKind::Atom(AtomicValue::Identifier {
                        name: identifier.to_owned(),
                        is_assignment: false
                    }),
                    0..identifier.len(),
                )
            );
        }
        test_identifier("foo");
        test_identifier("foo_bar");
        test_identifier("_foo_bar");
        test_identifier("_foo_bar_");
    }
}
