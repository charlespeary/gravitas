use std::vec::IntoIter;

use anyhow::{anyhow, Error, Result};

pub use crate::{
    parser::{
        expr::Expr,
        stmt::Stmt,
        token::{Affix, Token},
    },
    utils::{
        iter::{peek_nth, PeekNth},
        Either,
    },
};

mod error;
pub mod expr;
pub mod stmt;
mod token;

#[derive(Debug, PartialEq)]
pub struct Ast(pub Vec<Stmt>);

pub struct Parser {
    errors: Vec<Error>,
    tokens: PeekNth<IntoIter<Token>>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            errors: Vec::new(),
            tokens: peek_nth(tokens.into_iter()),
        }
    }

    // UTILITIES

    pub fn expect(&mut self, expected: Token) -> Result<Token> {
        let next = self.peek_token().clone();

        if next == expected {
            self.next_token();
            Ok(next)
        } else {
            Err(anyhow!("Expected {} but got {:#?}", expected, next))
        }
    }

    pub fn parse(mut self) -> Result<Ast, Vec<Error>> {
        let mut stmts: Vec<Stmt> = vec![];
        while self.tokens.peek().is_some() {
            match self.stmt() {
                Ok(stmt) => {
                    stmts.push(stmt);
                }
                Err(e) => {
                    self.errors.push(e);

                    while let Some(token) = self.tokens.peek() {
                        if !token.is_stmt() {
                            self.next_token();
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        if self.errors.is_empty() {
            // Global block wrapping all statements
            Ok(Ast(stmts))
        } else {
            Err(self.errors)
        }
    }

    fn peek_bp(&mut self, affix: Affix) -> usize {
        self.tokens.peek().map_or(0, |t| t.bp(affix))
    }

    fn peek_eq(&mut self, expected: Token) -> bool {
        self.tokens.peek().map_or(false, |t| t == &expected)
    }

    fn peek_eq_many(&mut self, expected: &[Token]) -> bool {
        expected.contains(self.peek_token())
    }

    fn peek_token(&mut self) -> &Token {
        self.tokens.peek().expect("Tried to peek empty iterator")
    }

    fn next_token(&mut self) -> Token {
        self.tokens.next().expect("Tried to iterate empty iterator")
    }
}
