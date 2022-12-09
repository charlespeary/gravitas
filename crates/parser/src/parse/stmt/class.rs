use crate::{
    parse::{
        stmt::{Stmt, StmtKind},
        Parser, StmtResult,
    },
    token::{
        constants::{CLOSE_BRACKET, OPEN_BRACKET},
        Token,
    },
    utils::combine,
};

impl<'t> Parser<'t> {
    pub(super) fn parse_class_declaration(&mut self) -> StmtResult {
        let class_keyword = self.expect(Token::Class)?.span();
        let class_name = self.expect_identifier()?.slice.to_owned();
        let super_class_identifier = if self.peek() == Token::Inherit {
            self.expect(Token::Inherit)?;
            let super_class_name = self.expect_identifier()?.slice.to_owned();
            Some(super_class_name)
        } else {
            None
        };

        self.expect(OPEN_BRACKET)?;

        let mut methods = Vec::new();

        loop {
            let next = self.peek();
            if next == CLOSE_BRACKET || next != Token::Function {
                break;
            }

            if self.peek() == Token::Function {
                let method = self.parse_fun_declaration()?;
                methods.push(method);
            }
        }

        let close_bracket = self.expect(CLOSE_BRACKET)?;

        Ok(Stmt::boxed(
            StmtKind::ClassDeclaration {
                name: class_name,
                super_class: super_class_identifier,
                methods,
            },
            combine(&class_keyword, &close_bracket.span()),
        ))
    }
}

#[cfg(test)]
mod test {

    use crate::{
        parse::{
            expr::{atom::AtomicValue, Expr, ExprKind},
            pieces::{Param, Params},
            stmt::{Stmt, StmtKind},
            Parser,
        },
        token::constants::{CLOSE_BRACKET, OPEN_BRACKET},
        utils::{
            error::{Expect, ParseErrorCause},
            test::parser::DUMMY_SPAN,
        },
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
                name: "foo".to_owned(),
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
                    name: "foo".to_owned(),
                    super_class: Some("bar".to_owned()),
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
                            name: "bar".to_owned(),
                            params: Params::new(
                                vec![
                                    Param::new("a".to_owned(), DUMMY_SPAN),
                                    Param::new("b".to_owned(), DUMMY_SPAN),
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
                    name: "foo".to_owned(),
                    super_class: None,
                },
                DUMMY_SPAN,
            ),
        );
    }
}
