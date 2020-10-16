use anyhow::{anyhow, Result};
use std::convert::TryFrom;

use crate::parser::{Affix, Expr, Parser, Token};

pub(crate) enum Operator {
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

impl TryFrom<Token> for Operator {
    fn from(token: Token) -> Result<Self> {
        Ok(match token {
            Token::Plus => Operator::Plus,
            Token::Minus => Operator::Minus,
            Token::Star => Operator::Multiply,
            Token::Divide => Operator::Divide,
            Token::Assign => Operator::Assign,
            Token::Equal => Operator::Compare,
            Token::Less => Operator::Less,
            Token::LessEqual => Operator::LessEqual,
            Token::Greater => Operator::Greater,
            Token::GreaterEqual => Operator::GreaterEqual,
            Token::Bang => Operator::Bang,
            _ => return Err(anyhow!("Tried to convert {} into operator.", token))
        })
    }
}

pub(crate) struct Binary {
    left: Box<Expr>,
    operator: Operator,
    right: Box<Expr>,
}

impl Into<Expr> for Binary {
    fn into(self) -> Expr {
        Expr::Binary(self)
    }
}

impl Parser {
    pub fn parse_binary(&mut self, left: Expr, rbp: usize) -> Result<Binary> {
        let operator: Operator = self.next_token().into()?;
        let right = self.parse_expr()?;

        Ok(Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }
}