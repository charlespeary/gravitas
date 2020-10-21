use anyhow::{anyhow, Result};

use crate::parser::{expr::Expr, operator::Operator, Parser, Token};

#[derive(Debug, Clone, PartialEq)]
pub struct Var {
    pub identifier: String,
    pub is_ref: bool,
}

impl Into<Expr> for Var {
    fn into(self) -> Expr {
        Expr::Var(self)
    }
}

impl Parser {
    pub fn parse_var(&mut self) -> Result<Var> {
        let token = self.next_token();
        if let Ok(identifier) = token.clone().into_identifier() {
            //    If next token is an assignment, then we are parsing binary expression
            //    In order to assign some value to variable in VM we're gonna need this to
            //    evaluate to variable's reference, not its value.
            let is_ref = self.peek_eq(Token::Operator(Operator::Assign));
            Ok(Var { identifier, is_ref })
        } else {
            Err(anyhow!("Expected variable identifier but got {}", token))
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn parse_var() {
        let mut parser = Parser::new(vec![Token::Identifier(String::from("variable"))]);

        assert_eq!(
            parser.parse_var().unwrap(),
            Var {
                is_ref: false,
                identifier: String::from("variable"),
            }
        )
    }
}
