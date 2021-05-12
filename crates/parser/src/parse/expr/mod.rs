use crate::common::error::Forbidden;
use crate::token::constants::{CLOSE_PARENTHESIS, OPEN_PARENTHESIS};
use crate::{
    common::{
        combine,
        error::{Expect, ParseErrorCause},
    },
    parse::{
        expr::atom::AtomicValue,
        operator::{BinaryOperator, UnaryOperator},
        stmt::Stmt,
        ExprResult, Node, Parser, Span, Symbol,
    },
    token::{operator::Operator, Token},
};
use std::convert::TryInto;
use std::fmt;
use std::fmt::Formatter;

pub(crate) mod atom;
pub(crate) mod control_flow;

pub(crate) type Expr = Node<Box<ExprKind>>;
pub(crate) type PathSegment = Node<Symbol>;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ExprKind {
    Atom(AtomicValue),
    Binary {
        lhs: Expr,
        op: Node<BinaryOperator>,
        rhs: Expr,
    },
    Unary {
        op: Node<UnaryOperator>,
        rhs: Expr,
    },
    Block {
        stmts: Vec<Stmt>,
        return_expr: Option<Expr>,
    },
    If {
        condition: Expr,
        body: Expr,
        else_expr: Option<Expr>,
    },
    While {
        condition: Expr,
        body: Expr,
    },
    Break {
        return_expr: Option<Expr>,
    },
    Continue,
    Call {
        callee: Expr,
        args: Vec<Expr>,
    },
    Index {
        target: Expr,
        position: Expr,
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
            _ => {
                write!(f, "NOT YET IMPLEMENTED!")?;
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
            Token::Operator(Operator::RoundBracketOpen) => {
                let open_paren = self.expect(OPEN_PARENTHESIS)?.span();
                let expr = self.parse_expression()?;
                let close_paren = self.expect(CLOSE_PARENTHESIS)?.span();
                Expr::new(expr.kind, combine(&open_paren, &close_paren))
            }
            Token::Operator(Operator::CurlyBracketOpen) => self.parse_block_expr()?,
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

        loop {
            let operator = match self.peek() {
                Token::Operator(operator) => operator,
                Token::Eof | Token::Semicolon | Token::Comma => break,
                _ => return Err(ParseErrorCause::UnexpectedToken),
            };

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
                continue;
            }

            let (l_bp, r_bp) = match operator.infix_bp() {
                Some(bp) => bp,
                _ => break,
            };

            if l_bp < min_bp {
                break;
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
}

#[cfg(test)]
mod test {
    use crate::common::test::parser::*;

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
        assert_expr("3 * (2 + 2)", "(* 3 (+ 2 2))");
        assert_expr("(3 * (2 + 2))", "(* 3 (+ 2 2))");
        assert_expr("3 + (3 * (2 + 2))", "(+ 3 (* 3 (+ 2 2)))");
    }

    #[test]
    fn parses_call_expression() {
        assert_expr("foo()", "$symbol()");
        assert_expr("foo(2)", "$symbol(2)");
        assert_expr("foo(2,3)", "$symbol(2,3)");
        assert_expr("foo() + bar()", "(+ $symbol() $symbol())");
    }
}
