use anyhow::{anyhow, Result};
use std::vec::IntoIter;

pub use crate::{
    parser::{
        ast::Ast,
        token::{Affix, Token},
        expr::Expr,
        stmt::Stmt,
        error::Error,
    }
};
use crate::parser::ast::{Arg, ExprOrStmt, FunctionType};
use crate::utils::{Either, peek_nth, PeekNth};

pub(crate) mod stmt;
pub(crate) mod ast;
mod error;
pub(crate) mod expr;
mod token;


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

//#[cfg(test)]
//mod test {
//    use super::*;
//
//    // EXPRESSIONS
//    // TODO: add tests for the error detection, error messages, struct helpers
//    #[test]
//    fn number_atom() {
//        let mut parser = Parser::new(vec![Token::Number(60.0)]);
//        assert_eq!(parser.expr(0).unwrap(), Expr::Atom(Atom::Number(60.0)));
//    }
//
//    #[test]
//    fn string_atom() {
//        let mut parser = Parser::new(vec![Token::Text(String::from("hello"))]);
//        assert_eq!(
//            parser.expr(0).unwrap(),
//            Expr::Atom(Atom::Text(String::from("hello")))
//        );
//    }
//
//    #[test]
//    fn grouping_expr() {
//        let mut parser = Parser::new(vec![
//            Token::OpenParenthesis,
//            Token::Number(10.0),
//            Token::CloseParenthesis,
//        ]);
//
//        assert_eq!(
//            parser.expr(0).unwrap(),
//            Expr::Grouping {
//                expr: Box::new(Expr::Atom(Atom::Number(10.0)))
//            }
//        )
//    }
//
//    #[test]
//    fn unary_expr() {
//        let mut parser = Parser::new(vec![
//            Token::Minus,
//            Token::OpenParenthesis,
//            Token::Number(10.0),
//            Token::CloseParenthesis,
//        ]);
//
//        assert_eq!(
//            parser.expr(0).unwrap(),
//            Expr::Unary {
//                operator: Token::Minus,
//                expr: Box::new(Expr::Grouping {
//                    expr: Box::new(Expr::Atom(Atom::Number(10.0)))
//                }),
//            }
//        )
//    }
//
//    #[test]
//    fn binary_expr() {
//        let mut parser = Parser::new(vec![Token::Number(10.0), Token::Plus, Token::Number(20.0)]);
//
//        assert_eq!(
//            parser.expr(0).unwrap(),
//            Expr::Binary {
//                left: Box::new(Expr::Atom(Atom::Number(10.0))),
//                operator: Token::Plus,
//                right: Box::new(Expr::Atom(Atom::Number(20.0))),
//            }
//        )
//    }
//
//    /// Assignment is just a simple binary operation
//    #[test]
//    fn binary_assignment_expr() {
//        let mut parser = Parser::new(vec![
//            Token::Identifier(String::from("var")),
//            Token::Assign,
//            Token::Number(20.0),
//        ]);
//
//        assert_eq!(
//            parser.expr(0).expect("Couldn't parse expression"),
//            Expr::Binary {
//                left: Box::new(Expr::Var {
//                    identifier: String::from("var"),
//                    is_ref: true,
//                }),
//                operator: Token::Assign,
//                right: Box::new(Expr::Atom(Atom::Number(20.0))),
//            }
//        )
//    }
//
//    #[test]
//    fn complicated_binary_expr() {
//        let mut parser = Parser::new(vec![
//            Token::OpenParenthesis,
//            Token::Number(-1.0),
//            Token::Plus,
//            Token::Number(2.0),
//            Token::CloseParenthesis,
//            Token::Star,
//            Token::Number(3.0),
//            Token::Minus,
//            Token::Number(-4.0),
//        ]);
//
//        assert_eq!(
//            parser.expr(0).unwrap(),
//            Expr::Binary {
//                left: Box::new(Expr::Binary {
//                    left: Box::new(Expr::Grouping {
//                        expr: Box::new(Expr::Binary {
//                            left: Box::new(Expr::Atom(Atom::Number(-1.0))),
//                            operator: Token::Plus,
//                            right: Box::new(Expr::Atom(Atom::Number(2.0))),
//                        })
//                    }),
//                    operator: Token::Star,
//                    right: Box::new(Expr::Atom(Atom::Number(3.0))),
//                }, ),
//                operator: Token::Minus,
//                right: Box::new(Expr::Atom(Atom::Number(-4.0))),
//            }
//        )
//    }
//
//    /// Parser uses Patt Parsing to determine the binding power of infix/prefix/postfix operators
//    /// so they are parsed in the correct order.
//    /// E.g 2 + 8 * 10 is parsed as Binary<2 + Binary<8 * 10>>, instead of Binary<10 * Binary<2 +8>>
//    #[test]
//    fn handle_binding_power() {
//        let mut parser = Parser::new(vec![
//            Token::Number(2.0),
//            Token::Plus,
//            Token::Number(8.0),
//            Token::Star,
//            Token::Number(10.0),
//        ]);
//
//        assert_eq!(
//            parser.expr(0).unwrap(),
//            Expr::Binary {
//                left: Box::new(Expr::Atom(Atom::Number(2.0))),
//                operator: Token::Plus,
//                right: Box::new(Expr::Binary {
//                    left: Box::new(Expr::Atom(Atom::Number(8.0))),
//                    operator: Token::Star,
//                    right: Box::new(Expr::Atom(Atom::Number(10.0))),
//                }),
//            }
//        )
//    }
//
//    #[test]
//    fn var_expr() {
//        let mut parser = Parser::new(vec![Token::Identifier(String::from("variable"))]);
//
//        assert_eq!(
//            parser.expr(0).unwrap(),
//            Expr::Var {
//                is_ref: false,
//                identifier: String::from("variable"),
//            }
//        )
//    }
//
//    #[test]
//    fn block_expr() {
//        let mut parser = Parser::new(vec![
//            Token::OpenBrace,
//            Token::Var,
//            Token::Identifier(String::from("var")),
//            Token::Assign,
//            Token::Number(10.0),
//            Token::Semicolon,
//            Token::CloseBrace,
//        ]);
//
//        assert_eq!(
//            parser
//                .block_expr()
//                .expect("Failed to parse block expression"),
//            Expr::Block {
//                body: Block {
//                    body: vec![Stmt::Var {
//                        identifier: String::from("var"),
//                        expr: Expr::Atom(Atom::Number(10.0)),
//                    }],
//                    final_expr: None,
//                }
//            }
//        )
//    }
//
//    #[test]
//    fn while_expr() {
//        let mut parser = Parser::new(vec![
//            Token::While,
//            Token::True,
//            Token::OpenBrace,
//            Token::Number(10.0),
//            Token::CloseBrace,
//        ]);
//        assert_eq!(
//            parser
//                .expr(0)
//                .expect("Unable to parse expression from given tokens."),
//            Expr::While {
//                condition: Box::new(Expr::Atom(Atom::Bool(true))),
//                body: Block {
//                    body: vec![],
//                    final_expr: Some(Box::new(Expr::Atom(Atom::Number(10.0)))),
//                },
//            }
//        );
//    }
//
//    // STATEMENTS
//
//    #[test]
//    fn print_stmt() {
//        assert_eq!(
//            Parser::new(vec![
//                Token::Print,
//                Token::Identifier(String::from("variable")),
//                Token::Semicolon,
//            ])
//                .stmt()
//                .unwrap(),
//            Stmt::Print {
//                expr: Expr::Var {
//                    is_ref: false,
//                    identifier: String::from("variable"),
//                }
//            }
//        );
//        assert_eq!(
//            Parser::new(vec![
//                Token::Print,
//                Token::Number(10.0),
//                Token::Plus,
//                Token::Number(20.0),
//                Token::Semicolon,
//            ])
//                .stmt()
//                .unwrap(),
//            Stmt::Print {
//                expr: Expr::Binary {
//                    left: Box::new(Expr::Atom(Atom::Number(10.0))),
//                    operator: Token::Plus,
//                    right: Box::new(Expr::Atom(Atom::Number(20.0))),
//                }
//            }
//        )
//    }
//
//    #[test]
//    fn var_stmt() {
//        assert_eq!(
//            Parser::new(vec![
//                Token::Var,
//                Token::Identifier(String::from("variable")),
//                Token::Assign,
//                Token::Number(10.0),
//                Token::Semicolon,
//            ])
//                .stmt()
//                .unwrap(),
//            Stmt::Var {
//                identifier: String::from("variable"),
//                expr: Expr::Atom(Atom::Number(10.0)),
//            }
//        );
//    }
//}
