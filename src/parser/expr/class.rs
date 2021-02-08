use anyhow::Result;

use crate::parser::{expr::Identifier, operator::Operator, Expr, Parser, Token};

/// Class related stuff like struct initializers
#[derive(Debug, PartialEq, Clone)]
pub struct StructInitializer {
    pub identifier: Identifier,
    pub properties: Vec<PropertyInitializer>,
}

impl Into<Expr> for StructInitializer {
    fn into(self) -> Expr {
        Expr::StructInitializer(self)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct PropertyInitializer {
    pub name: String,
    pub expr: Expr,
}

impl Parser {
    pub fn parse_struct_initializer(&mut self) -> Result<StructInitializer> {
        let identifier = self.parse_identifier()?;
        self.expect(Token::OpenBrace)?;

        let mut properties: Vec<PropertyInitializer> = vec![];
        while !self.peek_eq(Token::CloseBrace) {
            properties.push(self.parse_property_initializer()?);
        }
        self.expect(Token::CloseBrace)?;
        Ok(StructInitializer {
            properties,
            identifier,
        })
    }

    pub fn parse_property_initializer(&mut self) -> Result<PropertyInitializer> {
        let name = self.expect_identifier()?;
        self.expect(Token::Operator(Operator::Assign))?;
        let expr = self.parse_expr()?;
        Ok(PropertyInitializer { name, expr })
    }
}
