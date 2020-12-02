use anyhow::Result;

use crate::parser::{
    expr::{Block, Expr},
    Parser, Token,
};

#[derive(Debug, Clone, PartialEq)]
pub struct WhileLoop {
    pub condition: Box<Expr>,
    pub body: Block,
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

#[derive(Debug, Clone, PartialEq)]
pub struct Break {
    pub expr: Option<Box<Expr>>,
}

impl Into<Expr> for Break {
    fn into(self) -> Expr {
        Expr::Break(self)
    }
}

impl Parser {
    pub fn parse_optional_expr(&mut self) -> Result<Option<Expr>> {
        if !self.peek_eq(Token::Semicolon) & !self.at_end() {
            Ok(Some(self.parse_expr()?))
        } else {
            Ok(None)
        }
    }

    pub fn parse_break(&mut self) -> Result<Break> {
        self.expect(Token::Break)?;
        let expr = self.parse_optional_expr()?.map(Box::new);

        Ok(Break { expr })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Continue;

impl Into<Expr> for Continue {
    fn into(self) -> Expr {
        Expr::Continue(self)
    }
}

impl Parser {
    pub fn parse_continue(&mut self) -> Result<Continue> {
        self.expect(Token::Continue)?;
        Ok(Continue)
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::parser::expr::atom::Atom;

    use super::*;

    #[test]
    fn while_expr() {
        let mut parser = Parser::new(vec![
            Token::While,
            Token::True,
            Token::OpenBrace,
            Token::Number(10.0),
            Token::CloseBrace,
        ]);
        assert_eq!(
            parser
                .parse_while_loop()
                .expect("Unable to parse expression from given tokens."),
            WhileLoop {
                condition: Box::new(Expr::Atom(Atom::Bool(true))),
                body: Block {
                    body: vec![],
                    final_expr: Some(Box::new(Expr::Atom(Atom::Number(10.0)))),
                },
            }
        );
    }
}
