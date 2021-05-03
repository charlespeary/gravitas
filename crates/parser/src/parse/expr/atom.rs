use crate::{
    parse::{expr::Expr, Number, ParseResult, Parser, Spanned, Symbol},
    token::Token,
};
use derive_more::Display;

#[derive(Debug, Display, Clone, PartialEq)]
pub enum AtomicValue {
    Boolean(bool),
    Number(Number),
    #[display(fmt = "symbol")]
    Text(Symbol),
    #[display(fmt = "symbol")]
    Identifier(Symbol),
}

impl<'a> Parser<'a> {
    pub(super) fn parse_atom(&mut self) -> ParseResult<Expr> {
        let lexeme = self.advance()?;
        let span = lexeme.span();

        let val = match lexeme.token {
            Token::Bool(val) => AtomicValue::Boolean(val),
            Token::Number(val) => AtomicValue::Number(val),
            // It's safe to unwrap because these strings should be interned during advance()
            // If it panics then we have a bug in our code
            Token::String(_) => AtomicValue::Text(lexeme.intern_key.unwrap()),
            Token::Identifier(_) => AtomicValue::Identifier(lexeme.intern_key.unwrap()),
            t => {
                panic!("Encountered {:?} while parsing atom", t);
            }
        };

        Ok(Expr::Atom(Spanned { val, span }))
    }
}

#[cfg(test)]
pub(crate) mod test {
    use quickcheck_macros::quickcheck;

    use super::*;
    use lasso::Spur;

    #[test]
    fn parses_atom_booleans() {
        let mut parser = Parser::new("true false");
        assert_eq!(
            parser.parse_atom().unwrap(),
            Expr::Atom(Spanned {
                val: AtomicValue::Boolean(true),
                span: 0..4,
            })
        );
        assert_eq!(
            parser.parse_atom().unwrap(),
            Expr::Atom(Spanned {
                val: AtomicValue::Boolean(false),
                span: 5..10,
            })
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
            parser.parse_atom().unwrap(),
            Expr::Atom(Spanned {
                val: AtomicValue::Number(number),
                span: 0..num_as_str.len(),
            })
        );
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

        let parsed_string = parser.parse_atom().unwrap();

        assert_eq!(
            parsed_string,
            Expr::Atom(Spanned {
                val: AtomicValue::Text(Spur::default()),
                span: 0..text.len() + 2,
            })
        );
    }

    #[test]
    fn parses_atom_identifiers() {
        fn test_identifier(identifier: &str) {
            let mut parser = Parser::new(identifier);

            let parsed_identifier = parser.parse_atom().unwrap();
            assert_eq!(
                parsed_identifier,
                Expr::Atom(Spanned {
                    val: AtomicValue::Identifier(Spur::default()),
                    span: 0..identifier.len(),
                })
            );
        }
        test_identifier("foo");
        test_identifier("foo_bar");
        test_identifier("_foo_bar");
        test_identifier("_foo_bar_");
    }
}
