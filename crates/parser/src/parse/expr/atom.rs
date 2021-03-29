use std::ops::Range;

use crate::{
    parse::{ParseResult, Parser, VtasStringRef},
    token::Token,
};

#[derive(Debug, Clone)]
pub enum AtomicValue {
    Boolean(bool),
    Number(f64),
    Text(VtasStringRef),
    Identifier(VtasStringRef),
}

#[derive(Debug, Clone)]
pub(crate) struct Atom {
    val: AtomicValue,
    span: Range<usize>,
}

impl<'a> Parser<'a> {
    pub(super) fn parse_atom(&mut self) -> ParseResult<Atom> {
        let lexeme = self.advance()?;
        let span = lexeme.span();

        println!("{:?} {:?}", lexeme.token, lexeme.intern_key);

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
    use super::*;

    #[test]
    fn parser_parses_atoms() {
        // dbg!(test_parser!("false").parse_atom());
        // dbg!(test_parser!("true").parse_atom());
        // dbg!(test_parser!("2.0").parse_atom());
    }
}
