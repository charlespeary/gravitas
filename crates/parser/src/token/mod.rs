use lazy_static::lazy_static;
use logos::{Lexer, Logos};
use regex::Regex;

use operator::{lex_operator, Operator};

pub(crate) mod operator;

fn lex_number<'t>(lex: &mut Lexer<'t, Token<'t>>) -> Option<f64> {
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

fn lex_string<'t>(lex: &mut Lexer<'t, Token<'t>>) -> &'t str {
    let slice: &str = lex.slice();
    &slice[1..slice.len() - 1]
}

#[derive(Logos, Debug, PartialEq)]
pub(crate) enum Token<'t> {
    // KEYWORDS

    // OPERATORS
    #[regex(r"\+|\-|\*|/|%|\*\*|==|!=|<|<=|>|>=|or|and|!|\.|=", lex_operator)]
    Operator(Operator),
    // LITERALS
    #[regex("-?[0-9]*\\.?[0-9\\.]+", lex_number)]
    Number(f64),
    #[regex(r#""(\\"|[^"])*""#, lex_string)]
    String(&'t str),
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

    // #[quickcheck]
    #[test]
    fn lexer_parses_strings() {
        // dbg!(&format!("\"{}\"", text));
        // assert_token(&format!("\"{}\"", text), Token::String(&text));
        // Simple literals
        assert_token("\"foo\"", Token::String("foo"));
        // Literals with escapes
        assert_token(r#""fo\"o""#, Token::String(r#"fo\"o"#));
        // Empty strings
        assert_token(r#""""#, Token::String(""));
        assert_token(r#""    ""#, Token::String("    "));
    }

    #[quickcheck]
    fn q_lexer_parses_strings(text: String) {
        // Quickcheck generates strings with quotes and we don't allow these inside
        let text = text.replace("\"", "");
        assert_token(&format!("\"{}\"", text), Token::String(&text));
    }

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
