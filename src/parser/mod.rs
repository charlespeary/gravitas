use std::vec::IntoIter;

use anyhow::{anyhow, Context, Error, Result};
use logos::Logos;

pub use token::Token;

use crate::parser::ast::{Atom, Expr};
use crate::parser::token::AffixKind;
use crate::utils::{peek_nth, PeekNth};

mod ast;
mod token;

macro_rules! expect {
    ($self: ident, $token: path) => {{
        if !$self.peek_eq($token) {
            return Err(anyhow!("Expected {}!", $token));
        }
    }};
}

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
        println!("EXPR: {:#?}", expr);
    }

    fn peek_bp(&mut self, affix_kind: AffixKind) -> Result<usize> {
        self.tokens.peek().map_or(Ok(0), |t| t.bp(affix_kind))
    }

    fn peek_eq(&mut self, expected: Token) -> bool {
        self.tokens.peek().map_or(false, |t| t == &expected)
    }

    fn next_token(&mut self) -> Token {
        self.tokens.next().expect("Tried to iterate empty iterator")
    }

    fn expr(&mut self, rbp: usize) -> Result<Expr> {
        let mut expr = self.nud()?;
        while self.peek_bp(AffixKind::Infix)? > rbp {
            expr = self.led(expr)?;
        }
        Ok(expr)
    }

    fn nud(&mut self) -> Result<Expr> {
        let token = self.next_token();
        match token {
            // handle atoms
            Token::Text(text) => Ok(Expr::Atom(Atom::Text(text))),
            Token::Number(num) => Ok(Expr::Atom(Atom::Number(num))),
            // handle prefixes
            Token::Minus => self.unary(),
            Token::OpenParenthesis => self.grouping(),
            // handle unknown stuff
            _ => Err(anyhow!("Tried to turn invalid token into atom!")),
        }
    }

    fn grouping(&mut self) -> Result<Expr> {
        let open_paren = self.current_token();
        let bp = open_paren.bp(AffixKind::Prefix)?;
        let expr = self.expr(bp)?;

        expect!(self, Token::CloseParenthesis);

        Ok(Expr::Grouping {
            expr: Box::new(expr),
        })
    }

    fn unary(&mut self) -> Result<Expr> {
        let token = self.current_token();
        let bp = token.bp(AffixKind::Prefix)?;
        let expr = self.expr(bp)?;

        Ok(Expr::Unary {
            expr: Box::new(expr),
        })
    }

    fn led(&mut self, left: Expr) -> Result<Expr> {
        let operator = self.next_token();
        let right = self.expr(operator.bp(AffixKind::Infix)?)?;

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
    use anyhow::{Context, Error, Result};

    use crate::into_float;

    use super::*;

    // TODO: add tests for the error detection, error messages, struct helpers
    #[test]
    fn handles_numbers() {
        let mut parser = Parser::new(vec![Token::Number(into_float!(60.0))]);
        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Atom(Atom::Number(into_float!(60.0)))
        );
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
            Token::Number(into_float!(10.0)),
            Token::CloseParenthesis,
        ]);

        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Grouping {
                expr: Box::new(Expr::Atom(Atom::Number(into_float!(10.0))))
            }
        )
    }

    #[test]
    fn handles_unary() {
        let mut parser = Parser::new(vec![
            Token::Minus,
            Token::OpenParenthesis,
            Token::Number(into_float!(10.0)),
            Token::CloseParenthesis,
        ]);

        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Unary {
                expr: Box::new(Expr::Grouping {
                    expr: Box::new(Expr::Atom(Atom::Number(into_float!(10.0))))
                })
            }
        )
    }

    #[test]
    fn handles_binary() {
        let mut parser = Parser::new(vec![
            Token::Number(into_float!(10.0)),
            Token::Plus,
            Token::Number(into_float!(20.0)),
        ]);

        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Binary {
                left: Box::new(Expr::Atom(Atom::Number(into_float!(10.0)))),
                operator: Token::Plus,
                right: Box::new(Expr::Atom(Atom::Number(into_float!(20.0)))),
            }
        )
    }

    /// Parser uses Patt Parsing to determine the binding power of infix/prefix/postfix operators
    /// so they are parsed in the correct order.
    /// E.g 2 + 8 * 10 is parsed as Binary<2 + Binary<8 * 10>>, instead of Binary<10 * Binary<2 +8>>
    #[test]
    fn handle_binding_power() {
        let mut parser = Parser::new(vec![
            Token::Number(into_float!(2.0)),
            Token::Plus,
            Token::Number(into_float!(8.0)),
            Token::Star,
            Token::Number(into_float!(10.0)),
        ]);

        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Binary {
                left: Box::new(Expr::Atom(Atom::Number(into_float!(2.0)))),
                operator: Token::Plus,
                right: Box::new(Expr::Binary {
                    left: Box::new(Expr::Atom(Atom::Number(into_float!(8.0)))),
                    operator: Token::Star,
                    right: Box::new(Expr::Atom(Atom::Number(into_float!(10.0)))),
                }),
            }
        )
    }
}
