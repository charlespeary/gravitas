use anyhow::{anyhow, Result};
use enum_as_inner::EnumAsInner;

pub use crate::parser::{
    expr::{
        affix::Affix,
        atom::Atom,
        binary::Binary,
        block::Block,
        call::{Call, Return},
        conditional::If,
        grouping::Grouping,
        identifier::Identifier,
        loops::{Break, Continue, WhileLoop},
    },
    operator::Operator,
    Parser, Token,
};

pub mod affix;
pub mod atom;
pub mod binary;
pub mod block;
pub mod call;
pub mod conditional;
pub mod grouping;
pub mod identifier;
pub mod loops;

#[derive(Debug, Clone, PartialEq, EnumAsInner)]
pub enum Expr {
    Affix(Affix),
    Binary(Binary),
    Call(Call),
    Return(Return),
    Identifier(Identifier),
    Grouping(Grouping),
    Block(Block),
    If(If),
    While(WhileLoop),
    Break(Break),
    Continue(Continue),
    Atom(Atom),
}

#[macro_export]
macro_rules! expr {
    ($val: expr) => {
        Into::<Expr>::into($val)
    };
}

#[macro_export]
macro_rules! try_expr {
    ($val: expr) => {
        Into::<Expr>::into($val?)
    };
}

impl Parser {
    pub fn parse_expr(&mut self) -> Result<Expr> {
        Ok(self.parse_expr_bp(0)?)
    }

    pub fn parse_expr_bp(&mut self, min_bp: u8) -> Result<Expr> {
        let mut lhs = match self.peek_token()?.clone() {
            Token::Operator(operator) => {
                let ((), r_bp) = operator.prefix_bp()?;
                // advance operator
                self.next_token();
                Expr::Affix(Affix {
                    operator,
                    expr: Box::new(self.parse_expr_bp(r_bp)?),
                })
            }
            Token::While => try_expr!(self.parse_while_loop()),
            Token::OpenParenthesis => try_expr!(self.parse_grouping()),
            Token::OpenBrace => try_expr!(self.parse_block()),
            Token::If => try_expr!(self.parse_if()),
            Token::Identifier(_) => try_expr!(self.parse_identifier()),
            Token::Return => try_expr!(self.parse_return()),
            _ => try_expr!(self.parse_atom()),
        };

        if self.peek_eq(Token::OpenParenthesis) {
            lhs = self.parse_call(lhs)?.into();
        }
        // try to peek next token and see if it's an operator
        while let Some(op) = self.peek_next() {
            if let Ok(operator) = op
                .clone()
                .into_operator()
                .map_err(|_| anyhow!("Expected operator"))
            {
                if let Ok((l_bp, ())) = operator.postfix_bp() {
                    if l_bp < min_bp {
                        break;
                    }
                    self.next_token();
                    lhs = Expr::Affix(Affix {
                        operator,
                        expr: Box::new(lhs),
                    });
                    continue;
                }

                let (l_bp, r_bp) = operator.infix_bp()?;
                if l_bp < min_bp {
                    break;
                }
                self.next_token();

                // parse right side of the expression and turn the left side into binary
                let rhs = self.parse_expr_bp(r_bp)?;
                lhs = Expr::Binary(Binary {
                    lhs: Box::new(lhs),
                    operator,
                    rhs: Box::new(rhs),
                });
            } else {
                break;
            }
        }
        Ok(lhs)
    }
}
