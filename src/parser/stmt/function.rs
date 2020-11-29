use anyhow::{anyhow, Result};

use crate::parser::{
    expr::{Atom, Block, Expr},
    Parser,
    Stmt, stmt::var::VarStmt, Token,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub val: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionStmt {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Block,
}

impl Into<Stmt> for FunctionStmt {
    fn into(self) -> Stmt {
        Stmt::Function(self)
    }
}

impl Parser {
    /// Parse parameters. Used both for closures and function declarations.
    /// It expects the open parenthesis/brace, parses all of the identifiers and expects the close parenthesis/brace.
    /// Parameters must be followed by a coma.
    pub fn parse_params(&mut self) -> Result<Vec<Param>> {
        let mut params: Vec<Param> = vec![];

        while !self.peek_eq_many(&[Token::CloseParenthesis, Token::Bar])
            && self.peek_token().is_ok()
        {
            if let Ok(val) = self.next_token().into_identifier() {
                params.push(Param { val });

                // skip commas
                if self.peek_eq(Token::Coma) {
                    self.next_token();
                }
            } else {
                return Err(anyhow!("Expected argument. Received invalid token."));
            }
        }
        Ok(params)
    }

    /// Parse function declaration statement.
    /// It parses the function keyword, name, arguments and body.
    pub fn parse_function_stmt(&mut self) -> Result<FunctionStmt> {
        self.expect(Token::Function)?;
        if let Ok(name) = self.expect_identifier() {
            self.expect(Token::OpenParenthesis)?;
            let params = self.parse_params()?;
            self.expect(Token::CloseParenthesis)?;
            let body = self.parse_block()?;

            Ok(FunctionStmt { name, params, body })
        } else {
            Err(anyhow!("Expected function name!"))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! ident {
        ($ident: expr) => {
            Token::Identifier(String::from($ident))
        };
    }

    macro_rules! param {
        ($param: expr) => {
            Param {
                val: String::from($param),
            }
        };
    }

    #[test]
    fn parse_params() {
        let mut parser = Parser::new(vec![ident!("foo"), Token::Coma, ident!("bar")]);
        assert_eq!(
            parser.parse_params().unwrap(),
            vec![param!("foo"), param!("bar")]
        )
    }

    #[test]
    fn expect_function_name() {
        let mut parser = Parser::new(vec![Token::Function]);
        assert!(parser.parse_function_stmt().is_err())
    }
}
