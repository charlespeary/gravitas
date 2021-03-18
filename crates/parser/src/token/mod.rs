use lazy_static::lazy_static;
use logos::{Lexer, Logos};
use regex::Regex;

use operator::{lex_operator, Operator};

pub(crate) mod operator;

fn lex_number(lex: &mut Lexer<Token>) -> Option<f64> {
    lazy_static! {
        static ref MULTIPLE_DOTS_IN_NUMBER: Regex =
            Regex::new("(-|\\.)?[0-9]*((\\.[0-9]+){2,}|((\\.{2,}[0-9]*))|(([0-9]\\.){2,}))\\.?")
                .expect("Couldn't create regex(multiple dots in number)");
    }

    let slice: String = lex.slice().parse().ok()?;

    if MULTIPLE_DOTS_IN_NUMBER.is_match(&slice) {
        None
    } else {
        Some(
            slice
                .parse::<f64>()
                .expect("Couldn't parse f64 while lexing Token::Number"),
        )
    }
}

#[derive(Logos, Debug, PartialEq)]
pub(crate) enum Token {
    // KEYWORDS
    #[regex(r"\+|\-|\*|/|%|\*\*|==|!=|<|<=|>|>=|or|and|!|\.|=", lex_operator)]
    Operator(Operator),
    // LITERALS
    #[regex("-?[0-9]*\\.?[0-9\\.]+", lex_number)]
    Number(f64),
    #[error]
    #[regex(r"[ \t\n\f]+ |", logos::skip)]
    Error,
}

#[cfg(test)]
mod test {
    use quickcheck_macros::quickcheck;

    use crate::{
        common::test::{assert_error, assert_token},
        token::Token,
    };

    #[quickcheck]
    fn lexer_parses_numbers(number: f64) {
        // Ignore randomly generated stuff that can't be parsed.
        if number.is_nan() || number.is_infinite() {
            return;
        }
        assert_token(number.to_string().as_str(), Token::Number(number));
    }

    #[test]
    fn lexer_allows_numbers_with_trailing_commas() {
        use Token::Number;
        assert_token(".1", Number(0.1));
        assert_token("1.", Number(1.0));
    }

    #[test]
    fn lexer_discards_invalid_numbers() {
        // more than one dot at the beginning
        assert_error("..1");
        // more than one trailing dot
        assert_error("1..");
        assert_error("1.1.");
        // more than one dot inside number
        assert_error("1.1.1");
        assert_error("1.1.1.");
    }
}
