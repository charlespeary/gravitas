use derive_more::Display;
use std::ops::Range;

use crate::{
    parse::{expr::SExpr, ParseResult, Parser, VtasStringRef},
    token::Token,
};

#[derive(Debug, Clone, PartialEq)]
pub enum AtomicValue {
    Boolean(bool),
    Number(f64),

    Text(VtasStringRef),
    Identifier(VtasStringRef),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Atom {
    pub(crate) val: AtomicValue,
    pub(crate) span: Range<usize>,
}

impl Into<SExpr> for Atom {
    fn into(self) -> SExpr {
        SExpr::Atom(self)
    }
}

impl<'a> Parser<'a> {
    pub(super) fn parse_atom(&mut self) -> ParseResult<Atom> {
        let lexeme = self.advance()?;
        let span = lexeme.span();

        let val = match lexeme.token {
            Token::Bool(val) => AtomicValue::Boolean(val),
            Token::Number(val) => AtomicValue::Number(val),
            // It's safe to unwrap because these strings should be interned during advance()
            // If it panics then we have a bug in our code
            Token::String(_) => AtomicValue::Text(lexeme.intern_key.unwrap()),
            Token::Identifier(_) => AtomicValue::Identifier(lexeme.intern_key.unwrap()),
            _ => unreachable!(),
        };

        Ok(Atom { val, span })
    }
}

#[cfg(test)]
mod test {
    use quickcheck_macros::quickcheck;

    use super::*;

    #[test]
    fn parser_parses_atom_booleans() {
        let mut parser = Parser::new("true false");
        assert_eq!(
            parser.parse_atom().unwrap(),
            Atom {
                val: AtomicValue::Boolean(true),
                span: 0..4,
            }
        );
        assert_eq!(
            parser.parse_atom().unwrap(),
            Atom {
                val: AtomicValue::Boolean(false),
                span: 5..10,
            }
        )
    }

    #[quickcheck]
    fn parser_parses_atom_numbers(number: f64) {
        if number.is_nan() || number.is_infinite() {
            return;
        }

        let num_as_str = number.to_string();

        let mut parser = Parser::new(&num_as_str);

        assert_eq!(
            parser.parse_atom().unwrap(),
            Atom {
                val: AtomicValue::Number(number),
                span: 0..num_as_str.len(),
            }
        );
    }

    #[quickcheck]
    #[test]
    fn parser_parses_atom_strings(text: String) {
        let text = text.replace("\"", "");
        // Quote the string, so it's lexed as a string token and not an identifier
        // Also, get rid of the quotes because they are not allowed inside our string representation
        // and quickcheck generates those. It'd be a good idea to create our own implementation of that random string.
        let quoted_text = format!("\"{}\"", &text);
        let mut parser = Parser::new(&quoted_text);

        let interned_key = parser.intern(&text);

        let parsed_string = parser.parse_atom().unwrap();
        assert_eq!(
            parsed_string,
            Atom {
                val: AtomicValue::Text(interned_key),
                span: 0..text.len() + 2,
            }
        );

        // Interned key points to the same string that got interned before we parsed the atom
        if let AtomicValue::Text(key) = parsed_string.val {
            assert_eq!(parser.interner.resolve(&key), text);
        } else {
            // Panic if it's not there. Something went wrong.
            panic!();
        }
    }

    #[test]
    fn parser_parses_atom_identifiers() {
        fn test_identifier(identifier: &str) {
            let mut parser = Parser::new(identifier);

            let interned_key = parser.intern(&identifier);

            let parsed_identifier = parser.parse_atom().unwrap();
            assert_eq!(
                parsed_identifier,
                Atom {
                    val: AtomicValue::Identifier(interned_key),
                    span: 0..identifier.len(),
                }
            );

            // Interned key points to the same string that got interned before we parsed the atom
            if let AtomicValue::Identifier(key) = parsed_identifier.val {
                assert_eq!(parser.interner.resolve(&key), identifier);
            } else {
                // Panic if it's not there. Something went wrong.
                panic!();
            }
        }
        test_identifier("foo");
        test_identifier("foo_bar");
        test_identifier("_foo_bar");
        test_identifier("_foo_bar_");
    }
}
