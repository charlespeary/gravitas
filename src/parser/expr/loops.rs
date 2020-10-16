use anyhow::Result;

use crate::parser::{expr::{Block, Expr}, Parser, Token};

pub(crate) struct WhileLoop {
    condition: Expr,
    body: Block,
}

impl Into<Expr> for WhileLoop {
    fn into(self) -> Expr {
        Expr::While(self)
    }
}

impl Parser {
    pub fn parse_while_loop(&mut self) -> Result<WhileLoop> {
        self.expect(Token::While)?;
        let condition = Box::new(self.parse_expr()?);
        let body = self.parse_block()?;

        Ok(WhileLoop { condition, body })
    }
}


pub(crate) struct Break {
    expr: Option<Box<Expr>>
}

impl Into<Expr> for Break {
    fn into(self) -> Expr {
        Expr::Break(self)
    }
}

impl Parser {
    pub fn parse_optional_expr(&mut self) -> Result<Option<Expr>> {
        if !self.peek_eq(Token::Semicolon) {
            Ok(Some(self.parse_expr()?))
        } else {
            Ok(None)
        }
    }

    pub fn parse_break(&mut self) -> Result<WhileLoop> {
        self.expect(Token::Break)?;
        let expr = self.parse_optional_expr()?.map(Box::new);

        Ok(Break { expr })
    }
}

pub(crate) struct Continue;

impl Into<Expr> for Continue {
    fn into(self) -> Expr {
        Expr::Continue(self)
    }
}

impl Parser {
    pub fn parse_continue(&mut self) -> Result<WhileLoop> {
        self.expect(Token::Continue)?;
        Ok(Continue)
    }
}