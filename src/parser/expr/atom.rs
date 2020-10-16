use derive_more::Display;

use anyhow::{anyhow, Result};

use crate::parser::{expr::Expr, Parser, Token};

#[derive(Debug, Display, PartialEq)]
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
                return Err(anyhow!(
        "Expected atom but got: {}",
            token
            ));
            }
        })
    }
}