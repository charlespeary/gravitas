use crate::{
    common::{combine, error::ParseErrorCause},
    parse::{
        expr::atom::AtomicValue,
        operator::{BinaryOperator, UnaryOperator},
        stmt::Stmt,
        ExprResult, Parser, Span, Spanned,
    },
    token::{operator::Operator, Token},
};
use derive_more::Display;
use std::convert::TryInto;
use std::fmt;
use std::fmt::Formatter;

pub(crate) mod atom;
pub(crate) mod control_flow;

#[derive(Debug, Display, Clone, PartialEq)]
#[display(fmt = "{}", kind)]
pub(crate) struct Expr {
    pub(crate) kind: Box<ExprKind>,
    pub(crate) span: Span,
}

impl Expr {
    pub(crate) fn new(kind: ExprKind, span: Span) -> Self {
        Self {
            kind: Box::new(kind),
            span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ExprKind {
    Atom(AtomicValue),
    // #[display(fmt = "({} {} {})", op, lhs, rhs)]
    Binary {
        lhs: Expr,
        op: Spanned<BinaryOperator>,
        rhs: Expr,
    },
    // #[display(fmt = "({} {})", op, rhs)]
    Unary {
        op: Spanned<UnaryOperator>,
        rhs: Expr,
    },
    // #[display(fmt = "{{ {} {}}}", stmts, return_expr)]
    Block {
        stmts: Vec<Stmt>,
        return_expr: Option<Expr>,
    },
    // #[display(fmt = "{}[{}]", target, position)]
    Index {
        target: Expr,
        position: Expr,
    },
}

impl fmt::Display for ExprKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ExprKind::*;

        match self {
            Atom(value) => write!(f, "{}", value),
            Binary { lhs, op, rhs } => write!(f, "({} {} {})", op, lhs, rhs),
            Unary { op, rhs } => write!(f, "({} {})", op, rhs),
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

                Ok(())
            }
            _ => write!(f, ""),
        }
    }
}

impl<'t> Parser<'t> {
    pub(crate) fn parse_expression(&mut self) -> ExprResult {
        self.parse_expression_bp(0)
    }

    fn parse_expression_bp(&mut self, min_bp: u8) -> ExprResult {
        let mut lhs: Expr = match self.peek() {
            Token::Operator(Operator::CurlyBracketOpen) => self.parse_block_expr()?,
            Token::Operator(op) => {
                let ((), r_bp) = op.prefix_bp().ok_or(ParseErrorCause::ExpectedLiteral)?;
                let op = self.construct_spanned(op.try_into()?)?;
                let rhs = self.parse_expression_bp(r_bp)?;
                let range = combine(&op.span, &rhs.span);
                Expr::new(ExprKind::Unary { op, rhs }, range)
            }
            _ => self.parse_atom_expr()?,
        };

        loop {
            let operator = match self.peek() {
                Token::Operator(operator) => operator,
                Token::Eof | Token::Semicolon => break,
                _ => return Err(ParseErrorCause::UnexpectedToken),
            };

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
                Spanned {
                    val: operator.try_into()?,
                    span: lexeme.span(),
                }
            };

            let rhs = self.parse_expression_bp(r_bp)?;
            let span = combine(&lhs.span, &rhs.span);
            lhs = Expr::new(ExprKind::Binary { lhs, op, rhs }, span);
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
}
