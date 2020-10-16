use anyhow::{anyhow, Result};

use crate::parser::{ast::{ExprOrStmt, Visitable}, expr::Expr, Parser, stmt::{expr::ExprStmt, Stmt}, Token};
use crate::utils::Either;

#[derive(Debug, PartialEq)]
pub(crate) struct Block {
    pub body: Vec<Stmt>,
    pub final_expr: Option<Box<Expr>>,
}

impl Visitable for Block {}


impl Into<Expr> for Block {
    fn into(self) -> Expr {
        Expr::Block(self)
    }
}

impl Parser {
    fn parse_expr_or_stmt(&mut self) -> Result<ExprOrStmt> {
        match self.peek_token().is_stmt() {
            true => Ok(Either::Left(self.stmt()?)),
            false => {
                let expr = self.expr(0)?;
                if self.peek_eq(Token::Semicolon) {
                    self.expect(Token::Semicolon)?;
                    Ok(Either::Left(ExprStmt { expr }.into()))
                } else {
                    Ok(Either::Right(expr))
                }
            }
        }
    }

    pub fn parse_block(&mut self) -> Result<Block> {
        self.expect(Token::OpenBrace)?;

        let mut block_items: Vec<Either<Stmt, Expr>> = Vec::new();

        while !self.peek_eq(Token::CloseBrace) {
            block_items.push(self.parse_expr_or_stmt()?);
        }

        if !block_items.iter().rev().skip(1).all(|item| item.is_left()) {
            return Err(anyhow!(
                "Expressions are only allowed at the end of the block!"
            ));
        }

        // TODO: dirty WIP logic
        let final_expr = if block_items.last().map(|i| i.is_right()).unwrap_or(false) {
            block_items.pop().unwrap().into_right().ok().map(Box::new)
        } else {
            None
        };

        let body = block_items
            .into_iter()
            .map(|i| {
                i.into_left().map_err(|_| {
                    anyhow!("Standalone expressions in block are only supported at the end!")
                })
            })
            .collect::<Result<Vec<Stmt>>>()?;

        self.expect(Token::CloseBrace)?;

        Ok(Block { body, final_expr })
    }
}