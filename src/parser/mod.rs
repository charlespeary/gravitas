use std::vec::IntoIter;

use anyhow::{anyhow, Error, Result};

pub use token::Token;

pub use crate::parser::ast::{Atom, Expr, Visitable, Visitor};
use crate::parser::token::Affix;
use crate::utils::{peek_nth, PeekNth};

mod ast;
mod token;

macro_rules! expect {
    ($self: ident, $token: path) => {{
        if !$self.peek_eq($token) {
            return Err(anyhow!("Expected {}!", $token));
        } else {
            $self.next_token();
        }
    }};
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

    pub fn compile(&mut self) -> Result<Expr> {
        self.expr(0)
    }

    fn expect(&mut self, expected: Token, message: &'static str) -> Result<Token> {
        if self.peek_eq(expected) {
            Err(anyhow!(message))
        } else {
            Ok(self.next_token())
        }
    }

    fn peek_bp(&mut self, affix: Affix) -> Result<usize> {
        self.tokens.peek().map_or(Ok(0), |t| t.bp(affix))
    }

    fn peek_eq(&mut self, expected: Token) -> bool {
        self.tokens.peek().map_or(false, |t| t == &expected)
    }

    fn next_token(&mut self) -> Token {
        self.tokens.next().expect("Tried to iterate empty iterator")
    }

    fn expr(&mut self, rbp: usize) -> Result<Expr> {
        let mut expr = self.prefix()?;
        while self.peek_bp(Affix::Infix)? > rbp {
            expr = self.binary(expr)?;
        }
        Ok(expr)
    }

    fn prefix(&mut self) -> Result<Expr> {
        let token = self.next_token();
        match token {
            // handle atoms
            Token::Text(text) => Ok(Expr::Atom(Atom::Text(text))),
            Token::Number(num) => Ok(Expr::Atom(Atom::Number(num))),
            Token::False => Ok(Expr::Atom(Atom::Bool(false))),
            Token::True => Ok(Expr::Atom(Atom::Bool(true))),
            Token::Null => Ok(Expr::Atom(Atom::Null)),
            // handle prefixes
            Token::Minus => self.unary(),
            Token::OpenParenthesis => self.grouping(),
            Token::Bang => self.unary(),
            // handle unknown stuff
            _ => Err(anyhow!("Tried to turn invalid token into atom!")),
        }
    }

    fn grouping(&mut self) -> Result<Expr> {
        let open_paren = self.current_token();
        let bp = open_paren.bp(Affix::Prefix)?;
        let expr = self.expr(bp)?;

        expect!(self, Token::CloseParenthesis);

        Ok(Expr::Grouping {
            expr: Box::new(expr),
        })
    }

    fn unary(&mut self) -> Result<Expr> {
        let token = self.current_token();
        let bp = token.bp(Affix::Prefix)?;
        let expr = self.expr(bp)?;

        Ok(Expr::Unary {
            expr: Box::new(expr),
        })
    }

    fn binary(&mut self, left: Expr) -> Result<Expr> {
        let operator = self.next_token();
        let right = self.expr(operator.bp(Affix::Infix)?)?;

        Ok(Expr::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    fn current_token(&self) -> &Token {
        self.tokens
            .current
            .as_ref()
            .expect("Tried to access token too early!")
    }
}

#[cfg(test)]
mod test {
    use anyhow::{Error, Result};

    use super::*;

    // TODO: add tests for the error detection, error messages, struct helpers
    #[test]
    fn handles_numbers() {
        let mut parser = Parser::new(vec![Token::Number(60.0)]);
        assert_eq!(parser.expr(0).unwrap(), Expr::Atom(Atom::Number(60.0)));
    }

    #[test]
    fn handles_strings() {
        let mut parser = Parser::new(vec![Token::Text(String::from("hello"))]);
        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Atom(Atom::Text(String::from("hello")))
        );
    }

    #[test]
    fn handles_grouping() {
        let mut parser = Parser::new(vec![
            Token::OpenParenthesis,
            Token::Number(10.0),
            Token::CloseParenthesis,
        ]);

        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Grouping {
                expr: Box::new(Expr::Atom(Atom::Number(10.0)))
            }
        )
    }

    #[test]
    fn handles_unary() {
        let mut parser = Parser::new(vec![
            Token::Minus,
            Token::OpenParenthesis,
            Token::Number(10.0),
            Token::CloseParenthesis,
        ]);

        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Unary {
                expr: Box::new(Expr::Grouping {
                    expr: Box::new(Expr::Atom(Atom::Number(10.0)))
                })
            }
        )
    }

    #[test]
    fn handles_binary() {
        let mut parser = Parser::new(vec![Token::Number(10.0), Token::Plus, Token::Number(20.0)]);

        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Binary {
                left: Box::new(Expr::Atom(Atom::Number(10.0))),
                operator: Token::Plus,
                right: Box::new(Expr::Atom(Atom::Number(20.0))),
            }
        )
    }

    #[test]
    fn handles_complicated_binary() {
        let mut parser = Parser::new(vec![
            Token::OpenParenthesis,
            Token::Number(-1.0),
            Token::Plus,
            Token::Number(2.0),
            Token::CloseParenthesis,
            Token::Star,
            Token::Number(3.0),
            Token::Minus,
            Token::Number(-4.0),
        ]);

        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Binary {
                left: Box::new(Expr::Binary {
                    left: Box::new(Expr::Grouping {
                        expr: Box::new(Expr::Binary {
                            left: Box::new(Expr::Atom(Atom::Number(1.0))),
                            operator: Token::Plus,
                            right: Box::new(Expr::Atom(Atom::Number(2.0))),
                        })
                    }),
                    operator: Token::Star,
                    right: Box::new(Expr::Atom(Atom::Number(3.0))),
                },),
                operator: Token::Minus,
                right: Box::new(Expr::Atom(Atom::Number(-4.0))),
            }
        )
    }

    /// Parser uses Patt Parsing to determine the binding power of infix/prefix/postfix operators
    /// so they are parsed in the correct order.
    /// E.g 2 + 8 * 10 is parsed as Binary<2 + Binary<8 * 10>>, instead of Binary<10 * Binary<2 +8>>
    #[test]
    fn handle_binding_power() {
        let mut parser = Parser::new(vec![
            Token::Number(2.0),
            Token::Plus,
            Token::Number(8.0),
            Token::Star,
            Token::Number(10.0),
        ]);

        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Binary {
                left: Box::new(Expr::Atom(Atom::Number(2.0))),
                operator: Token::Plus,
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::Atom(Atom::Number(8.0))),
                    operator: Token::Star,
                    right: Box::new(Expr::Atom(Atom::Number(10.0))),
                }),
            }
        )
    }
}
