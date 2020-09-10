use std::hint::unreachable_unchecked;
use std::iter::Fuse;
use std::slice::Iter;
use std::vec::IntoIter;

use anyhow::{anyhow, Context, Error, Result};
use logos::{Lexer, Logos};

pub use token::Token;

use crate::chunk::{Chunk, Opcode};
use crate::parser::ast::{Atom, Expr};
use crate::utils::{peek_nth, PeekNth};

mod ast;
mod token;

pub fn compile(code: &str) {
    println!("Going to lex: {}", code);
    // println!("{:#?}", Token::lexer("test").into_iter().collect::<Vec<_>>());
    let tokens: Vec<Token> = Token::lexer(code).collect();
    // println!("{:#?}", tokens);
    let mut parser = Parser::new(tokens);
    parser.compile();
}

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

    pub fn compile(&mut self) {
        let expr = self.expr(0);
    }

    fn peek_bp(&mut self) -> usize {
        self.tokens.peek().map_or(0, |t| t.bp())
    }

    fn expr(&mut self, rbp: usize) -> Result<Expr> {
        let mut expr = self.nud()?;
        while self.peek_bp() > rbp {
            expr = self.led(expr)?;
        }
        Ok(expr)
    }

    fn nud(&mut self) -> Result<Expr> {
        let token = self.tokens.next().unwrap();
        match token {
            Token::Number(num) => Ok(Expr::Atom(Atom::Number(num))),
            _ => Err(anyhow!("Tried to turn invalid token into atom!")),
        }
    }

    fn led(&mut self, left: Expr) -> Result<Expr> {
        let operator = self.tokens.next().unwrap();
        let right = self.expr(operator.bp())?;

        return Ok(Expr::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        });
    }

    fn current_token(&self) -> &Token {
        self.tokens
            .current
            .as_ref()
            .expect("Tried to access token too early!")
    }
}
