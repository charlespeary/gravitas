use anyhow::Result;

use crate::parser::{expr::{Atom, Block, Expr}, Parser, Token};

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum BranchType {
    If,
    ElseIf,
    Else,
}


impl From<BranchType> for Token {
    fn from(bt: BranchType) -> Token {
        match bt {
            BranchType::If => Token::If,
            BranchType::ElseIf => Token::ElseIf,
            BranchType::Else => Token::Else,
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) struct IfBranch {
    pub condition: Expr,
    pub body: Block,
    pub branch_type: BranchType,
}

pub(crate) struct If {
    branches: Vec<IfBranch>,
}

impl Into<Expr> for If {
    fn into(self) -> Expr {
        Expr::If(self)
    }
}

impl Parser {
    fn parse_if_branch(&mut self, branch_type: BranchType) -> Result<IfBranch> {
        self.expect(Token::from(branch_type))?;

        let condition = match branch_type {
            BranchType::Else => Expr::Atom(Atom::Bool(true)),
            _ => self.parse_expr()?
        };

        let body = self.parse_block()?;

        Ok(IfBranch {
            branch_type,
            condition,
            body,
        })
    }

    pub fn parse_if(&mut self) -> Result<If> {
        let mut branches: Vec<IfBranch> = vec![];

        branches.push(self.parse_if_branch(BranchType::If)?);

        while self.peek_eq(Token::ElseIf) {
            branches.push(self.parse_if_branch(BranchType::ElseIf)?);
        }

        if self.peek_eq(Token::Else) {
            branches.push(self.parse_if_branch(BranchType::Else)?);
        }

        Ok(If { branches })
    }
}