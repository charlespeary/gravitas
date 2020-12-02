use anyhow::{anyhow, Result};

use crate::parser::{expr::Expr, Parser, Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    Text(String),
    Number(f64),
    Bool(bool),
    Null,
}

impl Into<Expr> for Atom {
    fn into(self) -> Expr {
        Expr::Atom(self)
    }
}

impl Parser {
    pub fn parse_atom(&mut self) -> Result<Atom> {
        let token = self.next_token();
        Ok(match token {
            Token::Text(text) => Atom::Text(text),
            Token::Number(num) => Atom::Number(num),
            Token::False => Atom::Bool(false),
            Token::True => Atom::Bool(true),
            Token::Null => Atom::Null,
            _ => {
                return Err(anyhow!("Expected atom but got: {}", token));
            }
        })
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parse_number_atom() {
        let mut parser = Parser::new(vec![Token::Number(60.0)]);
        assert_eq!(parser.parse_atom().unwrap(), Atom::Number(60.0));
    }

    #[test]
    fn parse_text_atom() {
        let mut parser = Parser::new(vec![Token::Text(String::from("hello"))]);
        assert_eq!(
            parser.parse_atom().unwrap(),
            Atom::Text(String::from("hello"))
        );
    }

    #[test]
    fn parse_true_bool_atom() {
        let mut parser = Parser::new(vec![Token::True]);
        assert_eq!(parser.parse_atom().unwrap(), Atom::Bool(true));
    }

    #[test]
    fn parse_false_bool_atom() {
        let mut parser = Parser::new(vec![Token::False]);
        assert_eq!(parser.parse_atom().unwrap(), Atom::Bool(false));
    }

    #[test]
    fn parse_null_atom() {
        let mut parser = Parser::new(vec![Token::Null]);
        assert_eq!(parser.parse_atom().unwrap(), Atom::Null);
    }

    #[test]
    fn parse_invalid_atom() {
        let mut parser = Parser::new(vec![Token::Function]);
        assert!(parser.parse_atom().is_err());
    }
}
