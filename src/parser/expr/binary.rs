use crate::{parser::Expr, parser::operator::Operator};

#[derive(Debug, Clone, PartialEq)]
pub struct Binary {
    pub lhs: Box<Expr>,
    pub operator: Operator,
    pub rhs: Box<Expr>,
}

impl Into<Expr> for Binary {
    fn into(self) -> Expr {
        Expr::Binary(self)
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::parser::{
        expr::{atom::Atom, Expr, Grouping, Identifier},
        Parser, Token,
    };

    use super::*;

    #[test]
    fn binary_expr() {
        let mut parser = Parser::new(vec![
            Token::Number(10.0),
            Token::Operator(Operator::Plus),
            Token::Number(20.0),
        ]);

        assert_eq!(
            parser.parse_expr().unwrap(),
            Expr::Binary(Binary {
                lhs: Box::new(Expr::Atom(Atom::Number(10.0))),
                operator: Operator::Plus,
                rhs: Box::new(Expr::Atom(Atom::Number(20.0))),
            })
        )
    }

    /// Assignment is just a simple binary operation
    #[test]
    fn binary_assignment_expr() {
        let mut parser = Parser::new(vec![
            Token::Identifier(String::from("var")),
            Token::Operator(Operator::Assign),
            Token::Number(20.0),
        ]);

        assert_eq!(
            parser.parse_expr().expect("Couldn't parse expression"),
            Expr::Binary(Binary {
                lhs: Box::new(Expr::Identifier(Identifier {
                    value: String::from("var"),
                    is_ref: true,
                })),
                operator: Operator::Assign,
                rhs: Box::new(Expr::Atom(Atom::Number(20.0))),
            })
        )
    }

    #[test]
    fn complicated_binary_expr() {
        let mut parser = Parser::new(vec![
            Token::OpenParenthesis,
            Token::Number(-1.0),
            Token::Operator(Operator::Plus),
            Token::Number(2.0),
            Token::CloseParenthesis,
            Token::Operator(Operator::Multiply),
            Token::Number(3.0),
            Token::Operator(Operator::Minus),
            Token::Number(-4.0),
        ]);

        assert_eq!(
            parser.parse_expr().unwrap(),
            Expr::Binary(Binary {
                lhs: Box::new(Expr::Binary(Binary {
                    lhs: Box::new(Expr::Grouping(Grouping {
                        expr: Box::new(Expr::Binary(Binary {
                            lhs: Box::new(Expr::Atom(Atom::Number(-1.0))),
                            operator: Operator::Plus,
                            rhs: Box::new(Expr::Atom(Atom::Number(2.0))),
                        }))
                    })),
                    operator: Operator::Multiply,
                    rhs: Box::new(Expr::Atom(Atom::Number(3.0))),
                }), ),
                operator: Operator::Minus,
                rhs: Box::new(Expr::Atom(Atom::Number(-4.0))),
            })
        )
    }

    /// Parser uses Patt Parsing to determine the binding power of infix/prefix/postfix operators
    /// so they are parsed in the correct order.
    /// E.g 2 + 8 * 10 is parsed as Binary<2 + Binary<8 * 10>>, instead of Binary<10 * Binary<2 +8>>
    #[test]
    fn handle_binding_power() {
        let mut parser = Parser::new(vec![
            Token::Number(2.0),
            Token::Operator(Operator::Plus),
            Token::Number(8.0),
            Token::Operator(Operator::Multiply),
            Token::Number(10.0),
        ]);

        assert_eq!(
            parser.parse_expr().unwrap(),
            Expr::Binary(Binary {
                lhs: Box::new(Expr::Atom(Atom::Number(2.0))),
                operator: Operator::Plus,
                rhs: Box::new(Expr::Binary(Binary {
                    lhs: Box::new(Expr::Atom(Atom::Number(8.0))),
                    operator: Operator::Multiply,
                    rhs: Box::new(Expr::Atom(Atom::Number(10.0))),
                })),
            })
        )
    }
}
