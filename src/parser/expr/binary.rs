use std::convert::{TryFrom, TryInto};

use anyhow::{anyhow, Error, Result};

use crate::parser::Affix;
use crate::{
    bytecode::Opcode,
    parser::{Expr, Parser, Token},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Assign,
    Compare,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    BangEqual,
    Bang,
}

impl Into<Opcode> for Operator {
    fn into(self) -> Opcode {
        match self {
            Operator::Plus => Opcode::Add,
            Operator::Minus => Opcode::Subtract,
            Operator::Multiply => Opcode::Multiply,
            Operator::Divide => Opcode::Divide,
            Operator::Assign => Opcode::Assign,
            Operator::Compare => Opcode::Compare,
            Operator::Less => Opcode::Less,
            Operator::LessEqual => Opcode::LessEqual,
            Operator::Greater => Opcode::Greater,
            Operator::GreaterEqual => Opcode::GreaterEqual,
            Operator::BangEqual => Opcode::BangEqual,
            Operator::Bang => Opcode::Not,
        }
    }
}

impl TryFrom<Token> for Operator {
    type Error = Error;

    fn try_from(token: Token) -> Result<Self> {
        Ok(match token {
            Token::Plus => Operator::Plus,
            Token::Minus => Operator::Minus,
            Token::Star => Operator::Multiply,
            Token::Divide => Operator::Divide,
            Token::Assign => Operator::Assign,
            Token::Compare => Operator::Compare,
            Token::Less => Operator::Less,
            Token::LessEqual => Operator::LessEqual,
            Token::Greater => Operator::Greater,
            Token::GreaterEqual => Operator::GreaterEqual,
            Token::Bang => Operator::Bang,
            _ => return Err(anyhow!("Tried to convert {} into operator.", token)),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Operator,
    pub right: Box<Expr>,
}

impl Into<Expr> for Binary {
    fn into(self) -> Expr {
        Expr::Binary(self)
    }
}

impl Parser {
    pub fn parse_binary(&mut self, left: Expr) -> Result<Binary> {
        println!("BINARY. NEXT TOKEN: {:#?}", self.peek_token());
        let token = self.next_token();
        let bp = token.bp(Affix::Infix);
        let operator: Operator = token.try_into()?;
        let right = self.parse_expr_bp(bp)?;

        Ok(Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::{assert_eq, assert_ne};

    use crate::parser::{
        expr::{atom::Atom, Expr, Grouping, Var},
        Token,
    };

    use super::*;

    #[test]
    fn binary_expr() {
        let mut parser = Parser::new(vec![Token::Number(10.0), Token::Plus, Token::Number(20.0)]);

        assert_eq!(
            parser.parse_expr().unwrap(),
            Expr::Binary(Binary {
                left: Box::new(Expr::Atom(Atom::Number(10.0))),
                operator: Operator::Plus,
                right: Box::new(Expr::Atom(Atom::Number(20.0))),
            })
        )
    }

    /// Assignment is just a simple binary operation
    #[test]
    fn binary_assignment_expr() {
        let mut parser = Parser::new(vec![
            Token::Identifier(String::from("var")),
            Token::Assign,
            Token::Number(20.0),
        ]);

        assert_eq!(
            parser.parse_expr().expect("Couldn't parse expression"),
            Expr::Binary(Binary {
                left: Box::new(Expr::Var(Var {
                    identifier: String::from("var"),
                    is_ref: true,
                })),
                operator: Operator::Assign,
                right: Box::new(Expr::Atom(Atom::Number(20.0))),
            })
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
            parser.parse_expr().unwrap(),
            Expr::Binary(Binary {
                left: Box::new(Expr::Binary(Binary {
                    left: Box::new(Expr::Grouping(Grouping {
                        expr: Box::new(Expr::Binary(Binary {
                            left: Box::new(Expr::Atom(Atom::Number(-1.0))),
                            operator: Operator::Plus,
                            right: Box::new(Expr::Atom(Atom::Number(2.0))),
                        }))
                    })),
                    operator: Operator::Multiply,
                    right: Box::new(Expr::Atom(Atom::Number(3.0))),
                }),),
                operator: Operator::Minus,
                right: Box::new(Expr::Atom(Atom::Number(-4.0))),
            })
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
            parser.parse_expr().unwrap(),
            Expr::Binary(Binary {
                left: Box::new(Expr::Atom(Atom::Number(2.0))),
                operator: Operator::Plus,
                right: Box::new(Expr::Binary(Binary {
                    left: Box::new(Expr::Atom(Atom::Number(8.0))),
                    operator: Operator::Multiply,
                    right: Box::new(Expr::Atom(Atom::Number(10.0))),
                })),
            })
        )
    }
}
