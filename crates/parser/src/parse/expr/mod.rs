use common::ProgramText;

use crate::{
    parse::{
        expr::atom::AtomicValue,
        operator::{BinaryOperator, UnaryOperator},
        stmt::Stmt,
        ExprResult, Node, Params, Parser,
    },
    token::constants::{
        ASSIGN, CLOSE_PARENTHESIS, CLOSE_SQUARE, DOT, OPEN_PARENTHESIS, OPEN_SQUARE,
    },
    token::{operator::Operator, Token},
    utils::{
        combine,
        error::{Expect, Forbidden, ParseErrorCause},
    },
};
use std::fmt;
use std::fmt::Formatter;
use std::{convert::TryInto, fmt::write};

pub mod atom;
pub(crate) mod control_flow;

pub type Expr = Node<Box<ExprKind>>;
pub type PathSegment = Node<ProgramText>;

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    // 1, false, "foo", foo
    Atom(AtomicValue),
    // 2 + 2, foo <= 10
    Binary {
        lhs: Expr,
        op: Node<BinaryOperator>,
        rhs: Expr,
    },
    // -foo, !false
    Unary {
        op: Node<UnaryOperator>,
        rhs: Expr,
    },
    // { }, { 2 } , { let x = 10; } { let x = 10; 10 }
    Block {
        stmts: Vec<Stmt>,
        return_expr: Option<Expr>,
    },
    // if true { 10 } else { 15 }
    If {
        condition: Expr,
        body: Expr,
        else_expr: Option<Expr>,
    },
    // while true { }
    While {
        condition: Expr,
        body: Expr,
    },
    // break, break 5
    Break {
        return_expr: Option<Expr>,
    },
    // continue
    Continue,
    // foo(), bar(10, 10)
    Call {
        callee: Expr,
        args: Vec<Expr>,
    },
    Return {
        value: Option<Expr>,
    },
    // [], [1, 2, 3]
    Array {
        values: Vec<Expr>,
    },
    // foo[10]
    Index {
        target: Expr,
        position: Expr,
    },
    // foo.bar, foo.bar.property
    // The target is an expression because we are not limited
    // only to identifiers. We can also call methods on literals
    // e.g "foo".toUppercase()
    GetProperty {
        target: Expr,
        is_method_call: bool,
        identifier: Node<ProgramText>,
    },
    SetProperty {
        target: Expr,
        value: Expr,
        identifier: Node<ProgramText>,
    },
    ObjectLiteral {
        properties: Vec<(ProgramText, Expr)>,
    },
    // a = b
    Assignment {
        target: Expr,
        value: Expr,
    },
    // (a,b) => a + b
    // (a,b) => { }
    Closure {
        params: Params,
        body: Expr,
    },
}

impl fmt::Display for ExprKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ExprKind::*;

        match self {
            Atom(value) => {
                write!(f, "{}", value)?;
            }
            Binary { lhs, op, rhs } => {
                write!(f, "({} {} {})", op, lhs, rhs)?;
            }
            Unary { op, rhs } => {
                write!(f, "({} {})", op, rhs)?;
            }
            Block { stmts, return_expr } => {
                write!(f, "{{ ")?;
                for (index, stmt) in stmts.iter().enumerate() {
                    if index > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", stmt)?;
                }

                if let Some(expr) = return_expr {
                    if !stmts.is_empty() {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", expr)?;
                }
                write!(f, " }}")?;
            }
            If {
                condition,
                body,
                else_expr,
            } => {
                write!(f, "if {}", condition)?;
                write!(f, " {}", body)?;
                if let Some(expr) = else_expr {
                    write!(f, " else {}", expr)?;
                }
            }
            While { condition, body } => {
                write!(f, "while {} {}", condition, body)?;
            }
            Break { return_expr } => match return_expr {
                Some(expr) => {
                    write!(f, "break {}", expr)?;
                }
                None => {
                    write!(f, "break")?;
                }
            },
            Continue => {
                write!(f, "continue")?;
            }
            Call { callee, args } => {
                write!(f, "{}", callee)?;
                write!(f, "(")?;
                let count = args.len().saturating_sub(1);
                for (index, arg) in args.iter().enumerate() {
                    write!(f, "{}", arg)?;
                    if index < count {
                        write!(f, ",")?;
                    }
                }
                write!(f, ")")?;
            }
            Return { value } => match value {
                Some(value) => {
                    write!(f, "return {}", value)?;
                }
                None => {
                    write!(f, "return")?;
                }
            },
            Index { target, position } => {
                write!(f, "{}", target)?;
                write!(f, "[")?;
                write!(f, "{}", position)?;
                write!(f, "]")?;
            }
            Array { values } => {
                write!(f, "[")?;
                let count = values.len().saturating_sub(1);
                for (index, value) in values.iter().enumerate() {
                    write!(f, "{}", value)?;
                    if index < count {
                        write!(f, ",")?;
                    }
                }
                write!(f, "]")?;
            }
            GetProperty {
                target, identifier, ..
            } => {
                write!(f, "{}.{}", target.kind.to_string(), identifier)?;
            }
            SetProperty {
                target,
                value,
                identifier,
            } => {
                write!(
                    f,
                    "{}.{} = {}",
                    target.kind.to_string(),
                    identifier,
                    value.kind.to_string()
                )?;
            }
            Assignment { target, value } => {
                write!(f, "{} = {}", target, value)?;
            }
            Closure { params, body } => {
                let params_count = params.kind.len();
                write!(f, "|{}| => {}", params_count, body)?;
            }
            ObjectLiteral { properties } => {
                write!(f, "obj ")?;
                for (name, value) in properties {
                    write!(f, "{}:{}", name, value.to_string())?;
                }
                write!(f, " obj")?;
            }
        }
        Ok(())
    }
}

impl<'t> Parser<'t> {
    pub(crate) fn parse_expression(&mut self) -> ExprResult {
        self.parse_expression_bp(0)
    }

    fn parse_expression_bp(&mut self, min_bp: u8) -> ExprResult {
        if !self.peek().is_expr() {
            return Err(ParseErrorCause::Expected(Expect::Expression));
        }

        let mut lhs: Expr = match self.peek() {
            Token::If => self.parse_if_expr()?,
            Token::While => self.parse_while_expr()?,
            Token::Break => self.parse_break_expr()?,
            Token::Continue => self.parse_continue_expr()?,
            Token::Return => self.parse_return_expr()?,
            Token::New => self.parse_obj_literal(false)?,
            Token::Bar => self.parse_closure_expression()?,
            Token::Operator(Operator::RoundBracketOpen) => {
                let open_paren = self.expect(OPEN_PARENTHESIS)?.span();
                let expr = self.parse_expression()?;
                let close_paren = self.expect(CLOSE_PARENTHESIS)?.span();
                Expr::new(expr.kind, combine(&open_paren, &close_paren))
            }
            Token::Operator(Operator::CurlyBracketOpen) => self.parse_block_expr()?,
            Token::Operator(Operator::SquareBracketOpen) => self.parse_array_expr()?,
            Token::Operator(op) => {
                let ((), r_bp) = op
                    .prefix_bp()
                    .ok_or(ParseErrorCause::Expected(Expect::Literal))?;
                let op = self.construct_node(op.try_into()?)?;
                let rhs = self.parse_expression_bp(r_bp)?;
                let range = combine(&op.span, &rhs.span);
                Expr::boxed(ExprKind::Unary { op, rhs }, range)
            }
            _ => self.parse_atom_expr()?,
        };

        while let Token::Operator(operator) = self.peek() {
            if let Some((l_bp, ())) = operator.postfix_bp() {
                if l_bp < min_bp {
                    break;
                }

                // call expr
                if operator == Operator::RoundBracketOpen {
                    let open_parenthesis = self.expect(OPEN_PARENTHESIS)?.span();
                    let mut args: Vec<Expr> = Vec::new();
                    loop {
                        let next = self.peek();
                        if next == CLOSE_PARENTHESIS || !next.is_expr() {
                            break;
                        }
                        let arg = self.parse_expression()?;
                        args.push(arg);

                        if self.peek() == Token::Comma {
                            self.expect(Token::Comma)?;
                        }
                    }
                    let close_parenthesis = self.expect(CLOSE_PARENTHESIS)?.span();
                    lhs = Expr::boxed(
                        ExprKind::Call { callee: lhs, args },
                        combine(&open_parenthesis, &close_parenthesis),
                    );
                }

                if operator == Operator::SquareBracketOpen {
                    let start = self.expect(OPEN_SQUARE)?.span();
                    let index_position = self.parse_expression()?;
                    let end = self.expect(CLOSE_SQUARE)?.span();
                    lhs = Expr::boxed(
                        ExprKind::Index {
                            target: lhs,
                            position: index_position,
                        },
                        combine(&start, &end),
                    );
                }
                continue;
            }

            let (l_bp, r_bp) = match operator.infix_bp() {
                Some(bp) => bp,
                _ => break,
            };

            if l_bp < min_bp {
                break;
            }

            if operator == Operator::Dot {
                while self.peek() == DOT {
                    let dot = self.expect(DOT)?.span();
                    let identifier_lexeme = self.expect_identifier()?;
                    let identifier_span = identifier_lexeme.span();

                    let identifier = Node {
                        span: combine(&dot, &identifier_lexeme.span()),
                        kind: identifier_lexeme.slice.to_owned(),
                    };

                    let is_assignment = self.peek() == ASSIGN;

                    if is_assignment {
                        self.expect(ASSIGN)?;
                        let value = self.parse_expression()?;
                        let span = combine(&lhs.span, &value.span);
                        lhs = Expr::boxed(
                            ExprKind::SetProperty {
                                target: lhs,
                                value,
                                identifier,
                            },
                            span,
                        );
                    } else {
                        let is_method_call = self.peek() == OPEN_PARENTHESIS;
                        let span = combine(&lhs.span, &identifier_span);

                        lhs = Expr::boxed(
                            ExprKind::GetProperty {
                                target: lhs,
                                is_method_call,
                                identifier,
                            },
                            span,
                        );
                    }
                }
                continue;
            }

            if operator == Operator::Assign {
                self.expect(ASSIGN)?;
                let value = self.parse_expression()?;
                let span = combine(&lhs.span, &value.span);
                lhs = Expr::boxed(ExprKind::Assignment { target: lhs, value }, span);
                continue;
            }

            // Advance and construct spanned operator
            let op = {
                let lexeme = self.advance()?;
                Node {
                    kind: operator.try_into()?,
                    span: lexeme.span(),
                }
            };

            let rhs = self.parse_expression_bp(r_bp)?;
            let span = combine(&lhs.span, &rhs.span);
            lhs = Expr::boxed(ExprKind::Binary { lhs, op, rhs }, span);
        }

        Ok(lhs)
    }

    pub(super) fn parse_array_expr(&mut self) -> ExprResult {
        let start = self.expect(OPEN_SQUARE)?.span();
        let mut values: Vec<Expr> = Vec::new();

        loop {
            let next = self.peek();
            if next == CLOSE_SQUARE || !next.is_expr() {
                break;
            }

            let value = self.parse_expression()?;
            values.push(value);

            let next = self.peek();
            if next != CLOSE_SQUARE {
                self.expect(Token::Comma)?;
                if self.peek() == CLOSE_SQUARE {
                    return Err(ParseErrorCause::NotAllowed(Forbidden::TrailingComma));
                }
            }
        }

        let end = self.expect(CLOSE_SQUARE)?.span();

        Ok(Expr::boxed(
            ExprKind::Array { values },
            combine(&start, &end),
        ))
    }

    pub(super) fn parse_return_expr(&mut self) -> ExprResult {
        let return_keyword = self.expect(Token::Return)?.span();
        let value = if self.peek().is_expr() {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let span = if let Some(expr) = &value {
            combine(&return_keyword, &expr.span)
        } else {
            return_keyword
        };

        Ok(Expr::boxed(ExprKind::Return { value }, span))
    }

    pub(super) fn parse_closure_expression(&mut self) -> ExprResult {
        let params = self.parse_params()?;
        self.expect(Token::Arrow)?;
        let body = self.parse_expression()?;
        let span = combine(&params.span, &body.span);
        Ok(Expr::boxed(ExprKind::Closure { params, body }, span))
    }
}

#[cfg(test)]
mod test {
    use crate::utils::error::{Expect, Forbidden, ParseErrorCause};
    use crate::utils::test::parser::*;

    #[test]
    fn parses_simple_binary_expression() {
        assert_expr("1 + 2", "(+ 1 2)");
        assert_expr("1 - 2", "(- 1 2)");
        assert_expr("1 * 2", "(* 1 2)");
        assert_expr("1 / 2", "(/ 1 2)");
        assert_expr("1 % 2", "(% 1 2)");
        assert_expr("1 ** 2", "(** 1 2)");
        assert_expr("1 == 2", "(== 1 2)");
        assert_expr("1 != 2", "(!= 1 2)");
        assert_expr("1 < 2", "(< 1 2)");
        assert_expr("1 <= 2", "(<= 1 2)");
        assert_expr("1 > 2", "(> 1 2)");
        assert_expr("1 >= 2", "(>= 1 2)");
        assert_expr("1 or 2", "(or 1 2)");
        assert_expr("1 and 2", "(and 1 2)");
    }

    #[test]
    fn parses_binary_expressions_with_equal_precedence() {
        // logical
        assert_expr("1 or 2 or 3", "(or (or 1 2) 3)");
        assert_expr("1 and 2 and 3", "(and (and 1 2) 3)");
        // comparison, this will get discarded during static analysis,
        // but we want to ensure that parser doesn't surprise us
        assert_expr("1 == 2 == 3", "(== (== 1 2) 3)");
        assert_expr("1 != 2 != 3", "(!= (!= 1 2) 3)");
        assert_expr("1 < 2 < 3", "(< (< 1 2) 3)");
        assert_expr("1 <= 2 <= 3", "(<= (<= 1 2) 3)");
        assert_expr("1 > 2 > 3", "(> (> 1 2) 3)");
        assert_expr("1 >= 2 >= 3", "(>= (>= 1 2) 3)");
        // addition and subtraction
        assert_expr("1 + 2 + 3", "(+ (+ 1 2) 3)");
        assert_expr("1 + 2 + 3 + 4", "(+ (+ (+ 1 2) 3) 4)");
        assert_expr("1 + 2 - 3", "(- (+ 1 2) 3)");
        assert_expr("1 - 2 + 3", "(+ (- 1 2) 3)");
        // multiplication, division, modulo
        assert_expr("1 * 2 * 3", "(* (* 1 2) 3)");
        assert_expr("1 / 2 * 3", "(* (/ 1 2) 3)");
        assert_expr("1 * 2 / 3", "(/ (* 1 2) 3)");
        assert_expr("1 % 2 % 3", "(% (% 1 2) 3)");
        assert_expr("1 * 2 / 3 % 4", "(% (/ (* 1 2) 3) 4)");
        // exponent
        assert_expr("1 ** 2 ** 3", "(** (** 1 2) 3)");
    }

    #[test]
    fn parses_binary_expressions_with_bigger_precedence() {
        // logical operators precedes comparison
        assert_expr("1 and 2 < 3", "(and 1 (< 2 3))");
        assert_expr("1 < 2 and 3", "(and (< 1 2) 3)");
        assert_expr("1 or 2 < 3", "(or 1 (< 2 3))");
        assert_expr("1 < 2 or 3", "(or (< 1 2) 3)");
        // comparison precedes addition and subtraction
        assert_expr("1 + 2 > 3", "(> (+ 1 2) 3)");
        assert_expr("1 > 2 + 3", "(> 1 (+ 2 3))");
        assert_expr("1 > 2 - 3", "(> 1 (- 2 3))");
        assert_expr("1 - 2 > 3", "(> (- 1 2) 3)");
        // addition and subtraction precedes multiplication, division and modulo
        assert_expr("1 + 2 * 3", "(+ 1 (* 2 3))");
        assert_expr("1 * 2 + 3", "(+ (* 1 2) 3)");
        assert_expr("1 - 2 / 3", "(- 1 (/ 2 3))");
        assert_expr("1 / 2 - 3", "(- (/ 1 2) 3)");
        assert_expr("1 + 2 % 3", "(+ 1 (% 2 3))");
        assert_expr("1 % 2 - 3", "(- (% 1 2) 3)");
        // multiplication, division and modulo precedes exponent
        assert_expr("1 * 2 ** 3", "(* 1 (** 2 3))");
        assert_expr("1 ** 2 / 3", "(/ (** 1 2) 3)");
        assert_expr("1 % 2 ** 3", "(% 1 (** 2 3))");
    }

    #[test]
    fn parses_unary_expressions() {
        assert_expr("- -1", "(- -1)");
        assert_expr("- 2 + 2", "(- (+ 2 2))");
        assert_expr("!true", "(! true)");
        assert_expr("!!true", "(! (! true))");
        assert_expr("!!!true", "(! (! (! true)))");
        assert_expr("!!!!true", "(! (! (! (! true))))");

        assert_expr("--5", "(- -5)");
        assert_expr("---5", "(- (- -5))");
        assert_expr("----5", "(- (- (- -5)))");
    }

    #[test]
    fn parses_combined_expression() {
        assert_expr("!true == false", "(== (! true) false)");
        assert_expr("!!true == !false", "(== (! (! true)) (! false))");
        assert_expr("2 >= 10 + 3", "(>= 2 (+ 10 3))");
        assert_expr("2 + 2 ** 3 >= 10 + 3", "(>= (+ 2 (** 2 3)) (+ 10 3))");
        assert_expr("- -2 - -2", "(- (- -2 -2))");
    }

    #[test]
    fn parses_parenthesized_expression() {
        assert_expr("(2 + 2)", "(+ 2 2)");
        assert_expr("(((2)))", "2");
        assert_expr("3 * (2 + 2)", "(* 3 (+ 2 2))");
        assert_expr("(3 * (2 + 2))", "(* 3 (+ 2 2))");
        assert_expr("3 + (3 * (2 + 2))", "(+ 3 (* 3 (+ 2 2)))");
    }

    #[test]
    fn parses_call_expression() {
        assert_expr("foo()", "foo()");
        assert_expr("foo(2)", "foo(2)");
        assert_expr("foo(2,3)", "foo(2,3)");
        assert_expr("foo() + bar()", "(+ foo() bar())");
    }

    #[test]
    fn parses_index_expression() {
        assert_expr("foo[0]", "foo[0]");
        assert_expr("foo[1 + 2]", "foo[(+ 1 2)]");
    }

    #[test]
    fn parses_array_expression() {
        assert_expr("[]", "[]");
        assert_expr("[1,2,3]", "[1,2,3]");

        assert_expr_error(
            "[1,2,]",
            ParseErrorCause::NotAllowed(Forbidden::TrailingComma),
        );
    }

    #[test]
    fn parses_property_expression() {
        assert_expr("foo.bar", "foo.bar");
        assert_expr("foo.bar.property", "foo.bar.property");
        assert_expr("foo.bar.property.prop", "foo.bar.property.prop");

        assert_expr_error("foo.", ParseErrorCause::Expected(Expect::Identifier));
    }

    #[test]
    fn parses_assignment_expression() {
        assert_expr("a = b", "a = b");
        assert_expr("a = a + 1", "a = (+ a 1)");
    }

    #[test]
    fn parses_return_expression() {
        assert_expr("return", "return");
        assert_expr("return 5", "return 5");
        assert_expr("return 5 + 5", "return (+ 5 5)");
    }

    #[test]
    fn parses_closure_expression() {
        assert_expr("|| => 10", "|0| => 10");
        assert_expr("|a,b,c| => a + b + c", "|3| => (+ (+ a b) c)");
    }
}
