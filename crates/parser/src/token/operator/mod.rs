use logos::Lexer;

use crate::token::Token;

pub(crate) mod precedence;

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum Operator {
    // MATH
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Exponent,
    // COMPARISON
    Compare,
    BangCompare,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Or,
    And,
    // MISC
    In,
    Bang,
    Assign,
    Dot,
}

pub(crate) fn lex_operator<'t>(lex: &mut Lexer<'t, Token<'t>>) -> Option<Operator> {
    let slice: String = lex.slice().parse().ok()?;
    Some(match slice.as_str() {
        "+" => Operator::Plus,
        "-" => Operator::Minus,
        "*" => Operator::Multiply,
        "/" => Operator::Divide,
        "%" => Operator::Modulo,
        "**" => Operator::Exponent,
        "=" => Operator::Assign,
        "==" => Operator::Compare,
        "!=" => Operator::BangCompare,
        "<" => Operator::Less,
        "<=" => Operator::LessEqual,
        ">" => Operator::Greater,
        ">=" => Operator::GreaterEqual,
        "or" => Operator::Or,
        "and" => Operator::And,
        "in" => Operator::In,
        "!" => Operator::Bang,
        "." => Operator::Dot,
        _ => unreachable!(),
    })
}

#[cfg(test)]
#[macro_use]
mod test {
    use crate::{
        common::test::assert_token,
        token::{operator::Operator, Token},
    };

    macro_rules! op {
        ($variant: ident) => {
            Token::Operator(Operator::$variant)
        };
    }

    #[test]
    fn lex_all_operators() {
        assert_token("+", op!(Plus));
        assert_token("-", op!(Minus));
        assert_token("*", op!(Multiply));
        assert_token("/", op!(Divide));
        assert_token("%", op!(Modulo));
        assert_token("**", op!(Exponent));
        assert_token("=", op!(Assign));
        assert_token("==", op!(Compare));
        assert_token("!=", op!(BangCompare));
        assert_token("<", op!(Less));
        assert_token("<=", op!(LessEqual));
        assert_token(">", op!(Greater));
        assert_token(">=", op!(GreaterEqual));
        assert_token("or", op!(Or));
        assert_token("and", op!(And));
        assert_token("in", op!(In));
        assert_token("!", op!(Bang));
        assert_token(".", op!(Dot));
    }
}
