use anyhow::{anyhow, Result};

use crate::parser::{
    expr::{class::PropertyInitializer, Identifier},
    stmt::{function::FunctionStmt, var::VarStmt},
    Parser, Stmt, Token,
};

/// Statement used to declare class.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassStmt {
    pub name: String,
    pub property_initializers: Vec<PropertyInitializer>,
    pub methods: Vec<FunctionStmt>,
    pub superclass: Option<Identifier>,
}

impl Into<Stmt> for ClassStmt {
    fn into(self) -> Stmt {
        Stmt::ClassStmt(self)
    }
}

impl Parser {
    pub fn parse_class_stmt(&mut self) -> Result<ClassStmt> {
        self.expect(Token::Class)?;
        let name = self.expect_identifier()?;
        let superclass = if self.peek_eq_consume(Token::Inherit) {
            let name = self.parse_identifier()?;
            Some(name)
        } else {
            None
        };
        self.expect(Token::OpenBrace)?;

        let mut property_initializers: Vec<PropertyInitializer> = vec![];
        let mut methods: Vec<FunctionStmt> = vec![];

        while !self.peek_eq(Token::CloseBrace) {
            match self.peek_token()? {
                Token::Function => methods.push(self.parse_function_stmt()?),
                Token::Identifier(_) => {
                    property_initializers.push(self.parse_property_initializer()?)
                }
                not_allowed => {
                    return Err(anyhow!(
                        "{} is not allowed inside class declaration",
                        not_allowed
                    ))
                }
            }
        }

        self.expect(Token::CloseBrace)?;
        self.expect(Token::Semicolon)?;

        Ok(ClassStmt {
            name,
            superclass,
            property_initializers,
            methods,
        })
    }
}
