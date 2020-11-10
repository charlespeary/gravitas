use anyhow::{anyhow, Result};
use derive_more::Display;
use enum_as_inner::EnumAsInner;
use logos::Lexer;

use crate::parser::Token;

#[derive(Debug, Display, Clone, Copy, PartialEq, EnumAsInner)]
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
    Dot,
}

impl Operator {
    pub fn infix_bp(self) -> Result<(u8, u8)> {
        Ok(match self {
            Operator::Assign => (0, 1),
            Operator::Plus | Operator::Minus => (1, 2),
            Operator::Multiply | Operator::Divide => (3, 4),
            Operator::Less | Operator::LessEqual | Operator::Greater | Operator::GreaterEqual => {
                (5, 6)
            }
            Operator::Compare => (7, 8),
            Operator::Dot => (8, 7),
            _ => return Err(anyhow!("{:?} can't be used as an infix operator!", self)),
        })
    }

    pub fn prefix_bp(self) -> Result<((), u8)> {
        Ok(match self {
            Operator::Plus | Operator::Minus => ((), 5),
            _ => return Err(anyhow!("{:?} can't be used as a prefix operator!", self)),
        })
    }

    pub fn postfix_bp(self) -> Result<(u8, ())> {
        Ok(match self {
            _ => return Err(anyhow!("{:?} can't be used as a postfix operator!", self)),
        })
    }
}

pub fn lex_operator(lex: &mut Lexer<Token>) -> Option<Operator> {
    let slice: String = lex.slice().parse().ok()?;
    Some(match slice.as_str() {
        "+" => Operator::Plus,
        "-" => Operator::Minus,
        "*" => Operator::Multiply,
        "/" => Operator::Divide,
        "=" => Operator::Assign,
        "==" => Operator::Compare,
        "<" => Operator::Less,
        "<=" => Operator::LessEqual,
        ">" => Operator::Greater,
        ">=" => Operator::GreaterEqual,
        "!" => Operator::Bang,
        "!=" => Operator::BangEqual,
        "." => Operator::Dot,
        _ => unreachable!(),
    })
}
