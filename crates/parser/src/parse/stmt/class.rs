use crate::{
    common::combine,
    parse::{
        stmt::{Stmt, StmtKind},
        Parser, StmtResult,
    },
    token::{
        constants::{CLOSE_BRACKET, OPEN_BRACKET},
        Token,
    },
};

impl<'t> Parser<'t> {
    pub(super) fn parse_class_declaration(&mut self) -> StmtResult {
        let class_keyword = self.expect(Token::Class)?.span();
        let (name, _) = self.expect_identifier()?;
        let super_class = if self.peek() == Token::Inherit {
            self.expect(Token::Inherit)?;
            let (super_class_name, _) = self.expect_identifier()?;
            Some(super_class_name)
        } else {
            None
        };

        self.expect(OPEN_BRACKET)?;

        let mut methods = Vec::new();
        let mut properties = Vec::new();

        loop {
            let next = self.peek();
            if next == CLOSE_BRACKET || (next != Token::Let && next != Token::Function) {
                break;
            }

            if self.peek() == Token::Let {
                let property = self.parse_variable_declaration()?;
                properties.push(property);
            }

            if self.peek() == Token::Function {
                let method = self.parse_fun_declaration()?;
                methods.push(method);
            }
        }

        let close_bracket = self.expect(CLOSE_BRACKET)?;

        Ok(Stmt::boxed(
            StmtKind::ClassDeclaration {
                name,
                super_class,
                methods,
                properties,
            },
            combine(&class_keyword, &close_bracket.span()),
        ))
    }
}

#[cfg(test)]
mod test {

    use crate::{
        common::{
            error::{Expect, ParseErrorCause},
            test::parser::{symbol, DUMMY_SPAN},
        },
        parse::{
            expr::{atom::AtomicValue, Expr, ExprKind},
            pieces::{Param, Params},
            stmt::{Stmt, StmtKind},
            Parser,
        },
        token::constants::{CLOSE_BRACKET, OPEN_BRACKET},
    };

    fn assert_class_error(input: &str, expected: ParseErrorCause) {
        let mut parser = Parser::new(input);
        assert_eq!(parser.parse_class_declaration().unwrap_err(), expected)
    }

    fn assert_class(input: &str, expected: Stmt) {
        let mut parser = Parser::new(input);
        assert_eq!(parser.parse_class_declaration().unwrap(), expected)
    }

    #[test]
    fn parser_parses_class_declarations() {
        // Parse errors
        assert_class_error("class", ParseErrorCause::Expected(Expect::Identifier));
        assert_class_error(
            "class foo",
            ParseErrorCause::Expected(Expect::Token(OPEN_BRACKET)),
        );
        assert_class_error(
            "class foo {",
            ParseErrorCause::Expected(Expect::Token(CLOSE_BRACKET)),
        );
        assert_class_error(
            "class foo: {}",
            ParseErrorCause::Expected(Expect::Identifier),
        );

        // Syntax
        let simple_class_input = "class foo {}";
        let simple_class = Stmt::boxed(
            StmtKind::ClassDeclaration {
                methods: Vec::new(),
                properties: Vec::new(),
                name: symbol(0),
                super_class: None,
            },
            DUMMY_SPAN,
        );
        let mut parser = Parser::new(simple_class_input);
        assert_eq!(parser.parse_stmt().unwrap(), simple_class);

        assert_class(simple_class_input, simple_class);

        assert_class(
            "class foo: bar {}",
            Stmt::boxed(
                StmtKind::ClassDeclaration {
                    methods: Vec::new(),
                    properties: Vec::new(),
                    name: symbol(0),
                    super_class: Some(symbol(1)),
                },
                DUMMY_SPAN,
            ),
        );

        assert_class(
            "class foo {\
                            let x = 5;\
                            let y = 10;\
                  }
        ",
            Stmt::boxed(
                StmtKind::ClassDeclaration {
                    methods: Vec::new(),
                    properties: vec![
                        Stmt::boxed(
                            StmtKind::VariableDeclaration {
                                name: symbol(1),
                                expr: Expr::boxed(ExprKind::Atom(AtomicValue::Number(5.0)), 0..5),
                            },
                            DUMMY_SPAN,
                        ),
                        Stmt::boxed(
                            StmtKind::VariableDeclaration {
                                name: symbol(2),
                                expr: Expr::boxed(ExprKind::Atom(AtomicValue::Number(10.0)), 0..5),
                            },
                            DUMMY_SPAN,
                        ),
                    ],
                    name: symbol(0),
                    super_class: None,
                },
                DUMMY_SPAN,
            ),
        );

        assert_class(
            "class foo {\
                         fn bar(a, b) {

                        }
                  }
        ",
            Stmt::boxed(
                StmtKind::ClassDeclaration {
                    methods: vec![Stmt::boxed(
                        StmtKind::FunctionDeclaration {
                            name: symbol(1),
                            params: Params::new(
                                vec![
                                    Param::new(symbol(2), DUMMY_SPAN),
                                    Param::new(symbol(3), DUMMY_SPAN),
                                ],
                                DUMMY_SPAN,
                            ),
                            body: Expr::boxed(
                                ExprKind::Block {
                                    return_expr: None,
                                    stmts: Vec::new(),
                                },
                                DUMMY_SPAN,
                            ),
                        },
                        DUMMY_SPAN,
                    )],
                    properties: Vec::new(),
                    name: symbol(0),
                    super_class: None,
                },
                DUMMY_SPAN,
            ),
        );
    }
}
