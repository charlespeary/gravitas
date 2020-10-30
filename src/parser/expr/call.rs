use anyhow::Result;

use crate::parser::{expr::Expr, Parser, Token};

#[derive(Debug, Clone, PartialEq)]
pub struct Args(pub Vec<Expr>);

#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    pub caller: Box<Expr>,
    pub args: Args,
}

impl Into<Expr> for Call {
    fn into(self) -> Expr {
        Expr::Call(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Return {
    pub expr: Option<Box<Expr>>,
}

impl Into<Expr> for Return {
    fn into(self) -> Expr {
        Expr::Return(self)
    }
}

impl Parser {
    pub fn parse_args(&mut self) -> Result<Args> {
        let mut args: Vec<Expr> = vec![];

        loop {
            if self.peek_eq(Token::CloseParenthesis) {
                break;
            }
            args.push(self.parse_expr()?);
            if !self.peek_eq(Token::Coma) {
                break;
            } else {
                self.next_token();
            }
        }

        Ok(Args(args))
    }

    /// Parse function/method call.
    /// The incoming expr is an identifier or object method on which the call is performed.
    pub fn parse_call(&mut self, expr: Expr) -> Result<Call> {
        self.expect(Token::OpenParenthesis)?;
        let args = self.parse_args()?;
        self.expect(Token::CloseParenthesis)?;

        Ok(Call {
            caller: Box::new(expr),
            args,
        })
    }

    /// Parse return expression
    pub fn parse_return(&mut self) -> Result<Return> {
        self.expect(Token::Return)?;
        let expr = self.parse_optional_expr()?.map(Box::new);
        Ok(Return { expr })
    }
}

#[cfg(test)]
mod test {
    use crate::parser::expr::{Atom, Identifier};

    use super::*;

    fn call_expr() -> Expr {
        Expr::Identifier(Identifier {
            value: String::from("foo"),
            is_ref: false,
        })
    }

    // Trailing coma is not allowed after last argument
    #[test]
    fn no_trailing_coma() {
        let mut parser = Parser::new(vec![Token::Number(10.0), Token::Coma]);
        assert!(parser.parse_args().is_err());
    }

    // Parse arguments(expressions) separated by comas
    #[test]
    fn parse_args() {
        let mut parser = Parser::new(vec![
            Token::Number(10.0),
            Token::Coma,
            Token::Number(20.0),
            Token::Coma,
            Token::Number(30.0),
        ]);

        assert_eq!(
            parser.parse_args().unwrap(),
            Args(vec![
                Expr::Atom(Atom::Number(10.0)),
                Expr::Atom(Atom::Number(20.0)),
                Expr::Atom(Atom::Number(30.0))
            ])
        )
    }

    // Require parenthesis
    #[test]
    fn require_open_parenthesis() {
        let mut parser = Parser::new(vec![
            Token::Number(10.0),
            Token::Coma,
            Token::Number(20.0),
            Token::Coma,
            Token::Number(30.0),
        ]);
        assert!(parser.parse_call(call_expr()).is_err())
    }

    #[test]
    fn require_close_parenthesis() {
        let mut parser = Parser::new(vec![
            Token::OpenParenthesis,
            Token::Number(10.0),
            Token::Coma,
            Token::Number(20.0),
            Token::Coma,
            Token::Number(30.0),
        ]);
        assert!(parser.parse_call(call_expr()).is_err())
    }

    #[test]
    fn parse_return_with_expr() {
        let mut parser = Parser::new(vec![Token::Return, Token::Number(2.0)]);

        assert_eq!(
            parser.parse_return().unwrap(),
            Return {
                expr: Some(Box::new(Expr::Atom(Atom::Number(2.0))))
            }
        )
    }

    #[test]
    fn parse_return_without_expr() {
        let mut parser = Parser::new(vec![Token::Return]);

        assert_eq!(parser.parse_return().unwrap(), Return { expr: None })
    }
}
