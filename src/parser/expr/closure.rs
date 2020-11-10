use anyhow::Result;

use crate::{parser::{Parser, Expr, stmt::function::Param, Token}};

#[derive(Debug, PartialEq, Clone)]
pub struct Closure {
    pub body: Box<Expr>,
    pub params: Vec<Param>,
}

impl Into<Expr> for Closure {
    fn into(self) -> Expr {
        Expr::Closure(self)
    }
}

impl Parser {
    pub fn parse_closure(&mut self) -> Result<Closure> {
        self.expect(Token::Bar)?;
        let params = self.parse_params()?;
        self.expect(Token::Bar)?;
        self.expect(Token::Arrow)?;
        let body = Box::new(self.parse_expr()?);

        Ok(Closure {
            params,
            body,
        })
    }
}