use std::io::repeat;
use std::vec::IntoIter;

use anyhow::{anyhow, Error, Result};

pub use token::Token;

pub use crate::parser::ast::{Atom, Expr, Stmt, Visitable, Visitor};
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
    stmts: Vec<Stmt>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            errors: Vec::new(),
            stmts: Vec::new(),
            tokens: peek_nth(tokens.into_iter()),
        }
    }

    // UTILITIES

    pub fn parse(mut self) -> Result<Vec<Stmt>, Vec<Error>> {
        while self.tokens.peek().is_some() {
            match self.stmt() {
                Ok(stmt) => {
                    self.stmts.push(stmt);
                }
                Err(e) => {
                    self.errors.push(e);
                    // while !self.peek_token().is_stmt() {
                    //     self.next_token();
                    // }
                }
            }
        }

        if self.errors.is_empty() {
            Ok(self.stmts)
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

    fn peek_token(&mut self) -> &Token {
        self.tokens.peek().expect("Tried to peek empty iterator")
    }

    fn next_token(&mut self) -> Token {
        self.tokens.next().expect("Tried to iterate empty iterator")
    }

    // EXPRESSIONS

    fn expr(&mut self, rbp: usize) -> Result<Expr> {
        let mut expr = self.prefix()?;
        while self.peek_bp(Affix::Infix) > rbp {
            expr = self.binary_expr(expr)?;
        }
        Ok(expr)
    }

    fn prefix(&mut self) -> Result<Expr> {
        let token = self.peek_token();
        match token {
            Token::Minus => self.unary_expr(),
            Token::Bang => self.unary_expr(),
            Token::Identifier(_) => self.var_expr(),
            Token::OpenParenthesis => self.grouping_expr(),
            Token::OpenBrace => self.block_expr(),
            _ => self.atom_expr(),
        }
    }

    fn var_expr(&mut self) -> Result<Expr> {
        let token = self.next_token();
        if let Ok(identifier) = token.into_identifier() {
            Ok(Expr::Var { identifier })
        } else {
            Err(anyhow!("Invalid token got into var_expr call!"))
        }
    }

    fn atom_expr(&mut self) -> Result<Expr> {
        let token = self.next_token();
        match token {
            Token::Text(text) => Ok(Expr::Atom(Atom::Text(text))),
            Token::Number(num) => Ok(Expr::Atom(Atom::Number(num))),
            Token::False => Ok(Expr::Atom(Atom::Bool(false))),
            Token::True => Ok(Expr::Atom(Atom::Bool(true))),
            Token::Null => Ok(Expr::Atom(Atom::Null)),
            _ => Err(anyhow!(
                "This token is not supported by the parser: {}",
                token
            )),
        }
    }

    fn grouping_expr(&mut self) -> Result<Expr> {
        let open_paren = self.next_token();
        let bp = open_paren.bp(Affix::Prefix);
        let expr = self.expr(bp)?;

        expect!(self, Token::CloseParenthesis);

        Ok(Expr::Grouping {
            expr: Box::new(expr),
        })
    }

    fn unary_expr(&mut self) -> Result<Expr> {
        let operator = self.next_token();
        let bp = operator.bp(Affix::Prefix);
        let expr = self.expr(bp)?;

        Ok(Expr::Unary {
            expr: Box::new(expr),
            operator,
        })
    }

    fn binary_expr(&mut self, left: Expr) -> Result<Expr> {
        let operator = self.next_token();
        let right = self.expr(operator.bp(Affix::Infix))?;

        Ok(Expr::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    fn block_expr(&mut self) -> Result<Expr> {
        expect!(self, Token::OpenBrace);

        let mut body: Vec<Stmt> = Vec::new();

        while !self.peek_eq(Token::CloseBrace) {
            body.push(self.stmt()?);
        }

        expect!(self, Token::CloseBrace);

        Ok(Expr::Block { body })
    }

    // STATEMENTS

    fn stmt(&mut self) -> Result<Stmt> {
        match self.peek_token() {
            Token::Print => self.print_stmt(),
            Token::Var => self.var_stmt(),
            _ => self.expr_stmt(),
        }
    }

    fn print_stmt(&mut self) -> Result<Stmt> {
        let _token = self.next_token();
        let expr = self.expr(0)?;
        expect!(self, Token::Semicolon);
        Ok(Stmt::Print { expr })
    }

    fn expr_stmt(&mut self) -> Result<Stmt> {
        let expr = self.expr(0)?;
        let terminated = self.peek_eq(Token::Semicolon);
        if terminated {
            self.next_token();
        }
        Ok(Stmt::Expr { expr, terminated })
    }

    fn var_stmt(&mut self) -> Result<Stmt> {
        let _token = self.next_token();
        if let Ok(identifier) = self.next_token().into_identifier() {
            expect!(self, Token::Assign);
            let expr = self.expr(0)?;
            expect!(self, Token::Semicolon);
            Ok(Stmt::Var { expr, identifier })
        } else {
            Err(anyhow!("Something went wrong"))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // TODO: add tests for the error detection, error messages, struct helpers
    #[test]
    fn number_atom() {
        let mut parser = Parser::new(vec![Token::Number(60.0)]);
        assert_eq!(parser.expr(0).unwrap(), Expr::Atom(Atom::Number(60.0)));
    }

    #[test]
    fn string_atom() {
        let mut parser = Parser::new(vec![Token::Text(String::from("hello"))]);
        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Atom(Atom::Text(String::from("hello")))
        );
    }

    #[test]
    fn grouping_expr() {
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
    fn unary_expr() {
        let mut parser = Parser::new(vec![
            Token::Minus,
            Token::OpenParenthesis,
            Token::Number(10.0),
            Token::CloseParenthesis,
        ]);

        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Unary {
                operator: Token::Minus,
                expr: Box::new(Expr::Grouping {
                    expr: Box::new(Expr::Atom(Atom::Number(10.0)))
                }),
            }
        )
    }

    #[test]
    fn binary_expr() {
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
    fn complicated_binary_expr() {
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
                            left: Box::new(Expr::Atom(Atom::Number(-1.0))),
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

    #[test]
    fn var_expr() {
        let mut parser = Parser::new(vec![Token::Identifier(String::from("variable"))]);

        assert_eq!(
            parser.expr(0).unwrap(),
            Expr::Var {
                identifier: String::from("variable")
            }
        )
    }

    #[test]
    fn print_stmt() {
        assert_eq!(
            Parser::new(vec![
                Token::Print,
                Token::Identifier(String::from("variable")),
                Token::Semicolon,
            ])
            .stmt()
            .unwrap(),
            Stmt::Print {
                expr: Expr::Var {
                    identifier: String::from("variable")
                }
            }
        );
        assert_eq!(
            Parser::new(vec![
                Token::Print,
                Token::Number(10.0),
                Token::Plus,
                Token::Number(20.0),
                Token::Semicolon,
            ])
            .stmt()
            .unwrap(),
            Stmt::Print {
                expr: Expr::Binary {
                    left: Box::new(Expr::Atom(Atom::Number(10.0))),
                    operator: Token::Plus,
                    right: Box::new(Expr::Atom(Atom::Number(20.0))),
                }
            }
        )
    }

    #[test]
    fn var_stmt() {
        assert_eq!(
            Parser::new(vec![
                Token::Var,
                Token::Identifier(String::from("variable")),
                Token::Assign,
                Token::Number(10.0),
                Token::Semicolon,
            ])
            .stmt()
            .unwrap(),
            Stmt::Var {
                identifier: String::from("variable"),
                expr: Expr::Atom(Atom::Number(10.0)),
            }
        );
    }
}
