use std::vec::IntoIter;

use anyhow::{anyhow, Context, Error, Result};

pub use crate::{
    parser::{expr::Expr, stmt::Stmt, token::Token},
    utils::{
        iter::{peek_nth, PeekNth},
        Either,
    },
};

mod error;
pub mod expr;
pub mod operator;
pub mod stmt;
pub mod token;

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

    pub fn expect(&mut self, expected: Token) -> Result<Token> {
        let next = self
            .peek_next()
            .with_context(|| anyhow!("Expected {} but got unexpected end of input", expected))?
            .clone();

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

    fn peek_eq(&mut self, expected: Token) -> bool {
        self.tokens.peek().map_or(false, |t| t == &expected)
    }

    fn peek_eq_many(&mut self, expected: &[Token]) -> bool {
        self.peek_token().map_or(false, |t| expected.contains(t))
    }

    fn peek_next(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    fn peek_token(&mut self) -> Result<&Token> {
        self.peek_next().with_context(|| "Unexpected end of input")
    }

    fn next_token(&mut self) -> Token {
        self.tokens.next().expect("Tried to iterate empty iterator")
    }
}

#[cfg(test)]
mod test {
    use crate::parser::{expr::atom::Atom, operator::Operator, stmt::var::VarStmt};

    use super::*;

    // expect() should return an error if the next token isn't the one we expect.
    #[test]
    fn expect_fail() {
        let mut parser = Parser::new(vec![Token::Var]);
        assert!(parser.expect(Token::Function).is_err());
    }

    // expect() should advance the iterator if the next token is the one we expect and return this token.
    #[test]
    fn expect_advance() {
        let mut parser = Parser::new(vec![Token::Var]);
        assert_eq!(parser.tokens.index, 0);
        assert_eq!(parser.expect(Token::Var).unwrap(), Token::Var);
        assert_eq!(parser.tokens.index, 1);
    }

    // peek_eq() should peek the next token and compare it with the given token.
    // In contrast to expect() it doesn't advance the iterator if both tokens are the same.
    #[test]
    fn peek_eq() {
        let mut parser = Parser::new(vec![
            Token::Function,
            Token::Var,
            Token::Operator(Operator::Bang),
        ]);
        assert!(parser.peek_eq(Token::Function));
        assert_eq!(Token::Function, parser.next_token());
        assert!(parser.peek_eq(Token::Var));
        assert_eq!(Token::Var, parser.next_token());
        assert!(parser.peek_eq(Token::Operator(Operator::Bang)));
        assert_eq!(Token::Operator(Operator::Bang), parser.next_token());
    }

    // peek_eq_many works the same as the peek_eq(), but we provide the function with many tokens.
    #[test]
    fn peek_eq_many() {
        let mut parser = Parser::new(vec![Token::Function]);
        assert!(parser.peek_eq_many(&[Token::Function]));
        assert!(parser.peek_eq_many(&[Token::Function, Token::Break]));
        assert!(!parser.peek_eq_many(&[Token::Class, Token::Break]));
    }

    // peek_token peeks the next token and returns it. It doesn't advance the iterator.
    #[test]
    fn peek_token() {
        let mut parser = Parser::new(vec![Token::Function]);
        assert_eq!(parser.tokens.index, 0);
        assert_eq!(parser.peek_token().unwrap(), &Token::Function);
        assert_eq!(parser.tokens.index, 0);
        assert_eq!(parser.next_token(), Token::Function);
    }

    // next_token() advances the tokens iterator and return the next token.
    // It panics if the iterator is empty.
    #[test]
    fn next_token() {
        let mut parser = Parser::new(vec![Token::Function]);
        assert_eq!(parser.tokens.index, 0);
        assert_eq!(parser.next_token(), Token::Function);
        assert_eq!(parser.tokens.index, 1);
    }

    // next_token() should panic if the iterator is empty.
    #[test]
    #[should_panic]
    fn next_token_empty_iterator() {
        let mut parser = Parser::new(vec![]);
        assert_eq!(parser.tokens.index, 0);
        parser.next_token();
    }

    // parse() should return the AST from given tokens
    #[test]
    fn parse() {
        let mut parser = Parser::new(vec![
            Token::Var,
            Token::Identifier(String::from("foo")),
            Token::Operator(Operator::Assign),
            Token::Number(10.0),
            Token::Semicolon,
        ]);

        assert_eq!(
            parser.parse().unwrap(),
            Ast(vec![Stmt::Var(VarStmt {
                identifier: String::from("foo"),
                expr: Expr::Atom(Atom::Number(10.0)),
            })])
        )
    }

    // parse() should return vector of errors if it can't produce AST from given tokens.
    // In case of an error, the parser skips the tokens until it encounters new statement
    // and starts parsing it in order to find as many errors as possible.
    #[test]
    fn parse_error() {
        let mut parser = Parser::new(vec![Token::Var]);
        let result = parser.parse();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().len(), 1);
    }

    #[test]
    fn parse_error_many() {
        let mut parser = Parser::new(vec![Token::Var, Token::Var, Token::Var]);
        let result = parser.parse();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().len(), 3);
    }
}
