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

fn lex_boolean<'t>(lex: &mut Lexer<'t, Token<'t>>) -> bool {
    match lex.slice() {
        "true" => true,
        "false" => false,
        _ => unreachable!(),
    }
}

#[derive(Logos, Debug, PartialEq, Clone)]
pub(crate) enum Token<'t> {
    // DECLARATION KEYWORDS
    #[token("fn")]
    Function,
    #[token("class")]
    Class,
    #[token("let")]
    Let,
    // EXPRESSION KEYWORDS
    #[token("if")]
    If,
    #[token("else if")]
    ElseIf,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("return")]
    Return,
    #[token("for")]
    For,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    // PARENTHESIS
    #[token("{")]
    CurlyBracketOpen,
    #[token("}")]
    CurlyBracketClose,
    #[token("(")]
    RoundBracketOpen,
    #[token(")")]
    RoundBracketClose,
    #[token("[")]
    SquareBracketOpen,
    #[token("]")]
    SquareBracketClose,
    // OPERATORS
    #[regex(r"\+|\-|\*|/|%|\*\*|==|!=|<|<=|>|>=|or|and|in|!|\.|=", lex_operator)]
    Operator(Operator),
    // LITERALS
    #[regex("true|false", lex_boolean)]
    Bool(bool),
    #[regex("-?[0-9]*\\.?[0-9\\.]+", lex_number)]
    Number(f64),
    #[regex(r#""(\\"|[^"])*""#, lex_string)]
    String(&'t str),
    #[regex("[a-z_A-Z][a-z_A-Z0-9]*")]
    Identifier(&'t str),
    #[error]
    #[regex(r"[ \t\n\f]+|([0-9]+[a-z_A-Z]+)", logos::skip)]
    Error,
}

#[cfg(test)]
mod test {
    use quickcheck_macros::quickcheck;

    use crate::{
        common::test::{assert_error, assert_token, assert_tokens},
        token::Token,
    };

    #[test]
    fn lexer_tokenizes_strings() {
        use Token::String;
        // Simple literals
        assert_token("\"foo\"", String("foo"));
        // Literals with escapes
        assert_token(r#""fo\"o""#, String(r#"fo\"o"#));
        // Empty strings
        assert_token(r#""""#, String(""));
        assert_token(r#""    ""#, String("    "));
    }

    #[quickcheck]
    fn q_lexer_tokenizes_strings(text: String) {
        // Quickcheck generates strings with quotes, and we don't allow these inside
        let text = text.replace("\"", "");
        assert_token(&format!("\"{}\"", text), Token::String(&text));
    }

    #[quickcheck]
    fn lexer_tokenizes_numbers(number: f64) {
        // Ignore randomly generated stuff that can't be parsed.
        if number.is_nan() || number.is_infinite() {
            return;
        }
        assert_token(number.to_string().as_str(), Token::Number(number));
    }

    #[test]
    fn lexer_tokenizes_numbers_with_trailing_commas() {
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

    // TODO: Discard numbers in front of the identifier as an error during the lexing when positive lookaheads are added to the Logos
    #[test]
    fn lexer_tokenizes_identifiers() {
        use Token::Identifier;
        // allow underscore at the beginning
        assert_token("_foo", Identifier("_foo"));
        // allow underscore at the end
        assert_token("foo_", Identifier("foo_"));
        // allow underscore inside
        assert_token("foo_bar", Identifier("foo_bar"));
        // allow numbers inside
        assert_token("fo123o", Identifier("fo123o"));
        // allow numbers at the end
        assert_token("foo123", Identifier("foo123"));
        // allow a-Z
        assert_token(
            "abcdefghijklmnopqrstuvwxyz",
            Identifier("abcdefghijklmnopqrstuvwxyz"),
        );
        // allow A-Z
        assert_token(
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            Identifier("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        );
        // allow a-zA-Z
        assert_token(
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
            Identifier("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        )
    }

    #[test]
    fn lexer_tokenizes_bool() {
        use Token::Bool;

        assert_token("true", Bool(true));
        assert_token("false", Bool(false));
    }

    #[test]
    fn lexer_tokenizes_parenthesis() {
        use Token::*;
        assert_token("{", CurlyBracketOpen);
        assert_token("}", CurlyBracketClose);
        assert_token("(", RoundBracketOpen);
        assert_token(")", RoundBracketClose);
        assert_token("[", SquareBracketOpen);
        assert_token("]", SquareBracketClose);

        assert_tokens("{}", &[CurlyBracketOpen, CurlyBracketClose]);
        assert_tokens("}{", &[CurlyBracketClose, CurlyBracketOpen]);
        assert_tokens("()", &[RoundBracketOpen, RoundBracketClose]);
        assert_tokens(")(", &[RoundBracketClose, RoundBracketOpen]);
        assert_tokens("[]", &[SquareBracketOpen, SquareBracketClose]);
        assert_tokens("][", &[SquareBracketClose, SquareBracketOpen]);
    }

    #[test]
    fn lexer_tokenizes_keywords() {
        use Token::*;

        assert_token("fn", Function);
        assert_token("class", Class);
        assert_token("let", Let);
        assert_token("if", If);
        assert_token("else if", ElseIf);
        assert_token("else", Else);
        assert_token("while", While);
        assert_token("return", Return);
        assert_token("for", For);
        assert_token("break", Break);
        assert_token("continue", Continue);
    }

    #[test]
    fn lexer_keywords_are_key_sensitive() {
        use Token::Identifier;
        assert_token("fN", Identifier("fN"));
        assert_token("clASS", Identifier("clASS"));
        assert_token("lEt", Identifier("lEt"));
        assert_token("iF", Identifier("iF"));
        assert_tokens("ElsE If", &[Identifier("ElsE"), Identifier("If")]);
        assert_token("eLSe", Identifier("eLSe"));
        assert_token("wHILe", Identifier("wHILe"));
        assert_token("rETUrn", Identifier("rETUrn"));
        assert_token("FOr", Identifier("FOr"));
        assert_token("bREAk", Identifier("bREAk"));
        assert_token("cONTInue", Identifier("cONTInue"));
    }
}
