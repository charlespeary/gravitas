use anyhow::{anyhow, Result};

use crate::parser::{expr::Expr, operator::Operator, Parser, Token};

#[derive(Debug, Clone, PartialEq)]
// An identifier that points to variable, function or class.
pub struct Identifier {
    pub value: String,
    // Stuff accessed via dot operator. Eg. this.foo, this.foo.bar, Something.foo()
    pub properties: Vec<String>,
    pub is_ref: bool,
}

impl Into<Expr> for Identifier {
    fn into(self) -> Expr {
        Expr::Identifier(self)
    }
}

impl Parser {
    pub fn parse_identifier(&mut self) -> Result<Identifier> {
        let token = self.next_token();
        if let Ok(value) = token.clone().into_identifier() {
            let mut properties: Vec<String> = vec![];

            while self.peek_eq_consume(Token::Dot) {
                properties.push(self.expect_identifier()?);
            }

            //    If next token is an assignment, then we are parsing binary expression
            //    In order to assign some value to variable in VM we're gonna need this to
            //    evaluate to variable's reference, not its value.
            let is_ref = self.peek_eq(Token::Operator(Operator::Assign));
            Ok(Identifier {
                value,
                is_ref,
                properties,
            })
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
    fn parse_identifier() {
        let mut parser = Parser::new(vec![Token::Identifier(String::from("variable"))]);

        assert_eq!(
            parser.parse_identifier().unwrap(),
            Identifier {
                is_ref: false,
                value: String::from("variable"),
            }
        )
    }
}
