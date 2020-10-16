use anyhow::{Result, anyhow};

use crate::parser::{expr::Block, Parser, Stmt, Token};

pub(crate) struct Arg {
    val: String
}

pub(crate) struct FunctionStmt {
    name: String,
    args: Vec<Arg>,
    body: Block,
}

impl Into<Stmt> for FunctionStmt {
    fn into(self) -> Stmt {
        Stmt::Function(self)
    }
}

impl Parser {
    pub fn parse_args(&mut self) -> Result<Vec<Arg>> {
        let mut args: Vec<Arg> = vec![];

        while !self.peek_eq_many(&[Token::CloseParenthesis, Token::CloseBrace]) {
            if let Ok(val) = self.next_token().into_identifier() {
                args.push(Arg { val });

                // skip commas
                if self.peek_eq(Token::Coma) {
                    self.next_token();
                }
            } else {
                return Err(anyhow!("Expected argument. Received invalid token."));
            }
        }
        Ok(args)
    }

    pub fn parse_function_stmt(&mut self) -> Result<FunctionStmt> {
        let _token = self.next_token();
        if let Ok(name) = self.next_token().into_identifier() {
            self.expect(Token::OpenParenthesis)?;
            let args = self.parse_args()?;
            self.expect(Token::CloseParenthesis)?;
            let body = self.parse_block()?;
            Ok(FunctionStmt { name, args, body })
        } else {
            Err(anyhow!("Expected function name!"))
        }
    }
}