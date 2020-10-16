use anyhow::{anyhow, Result};

use crate::parser::{expr::Expr, Parser, Token};

pub(crate) struct Var {
    identifier: String,
    is_ref: bool,
}

impl Into<Expr> for Var {
    fn into(self) -> Expr {
        Expr::Var(self)
    }
}

impl Parser {
    fn parse_var(&mut self) -> Result<Var> {
        let token = self.next_token();
        if let Ok(identifier) = token.into_identifier() {
            //    If next token is an assignment, then we are parsing binary expression
            //    In order to assign some value to variable in VM we're gonna need this to
            //    evaluate to variable's reference, not its value.
            let is_ref = self.peek_eq(Token::Assign);
            Ok(Var { identifier, is_ref })
        } else {
            Err(anyhow!("Expected variable identifier but got {}", token))
        }
    }
}