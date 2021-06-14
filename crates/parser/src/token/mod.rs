use derive_more::Display;
use itertools::{peek_nth, PeekNth};
use lazy_static::lazy_static;
use logos::Span;
use logos::{Filter, Logos};
use regex::Regex;

use common::Symbol;
use operator::{lex_operator, Operator};

pub(crate) mod constants;
pub(crate) mod operator;

fn lex_number<'t>(lex: &mut logos::Lexer<'t, Token<'t>>) -> Result<f64, Token<'t>> {
    lazy_static! {
        static ref MULTIPLE_DOTS_IN_NUMBER: Regex =
            Regex::new("(-|\\.)?[0-9]*((\\.[0-9]+){2,}|((\\.{2,}[0-9]*))|(([0-9]\\.){2,}))\\.?")
                .expect("Couldn't create regex(multiple dots in number)");
    }

    let slice: &str = lex.slice();

    if slice == "Infinity" || slice == "inf" {
        return Ok(f64::INFINITY);
    }

    if slice == "NaN" {
        return Ok(f64::NAN);
    }

    if MULTIPLE_DOTS_IN_NUMBER.is_match(&slice) {
        Err(Token::Error)
    } else {
        slice.parse::<f64>().map_err(|_| Token::Error)
    }
}

fn lex_string<'t>(lex: &mut logos::Lexer<'t, Token<'t>>) -> &'t str {
    let slice: &str = lex.slice();
    &slice[1..slice.len() - 1]
}

fn lex_boolean<'t>(lex: &mut logos::Lexer<'t, Token<'t>>) -> bool {
    match lex.slice() {
        "true" => true,
        "false" => false,
        _ => unreachable!(),
    }
}

fn lex_error<'t>(lex: &mut logos::Lexer<'t, Token<'t>>) -> Filter<()> {
    lazy_static! {
        static ref TO_SKIP: Regex =
            Regex::new(r"[\n\f\r \t]+|//.*").expect("Couldn't create regex(tokens to skip)");
        static ref INVALID_IDENTIFIER: Regex =
            Regex::new(r"([0-9]+[a-z_A-Z]+)").expect("Couldn't create regex(invalid identifiers)");
    }

    let slice: &str = lex.slice();

    if TO_SKIP.is_match(slice) {
        Filter::Skip
    } else {
        Filter::Emit(())
    }
}

#[derive(Logos, Debug, PartialEq, Clone, Copy, Display)]
pub enum Token<'t> {
    // DECLARATION KEYWORDS
    #[token("fn")]
    Function,
    #[token("class")]
    Class,
    #[token("this")]
    This,
    #[token("super")]
    Super,
    #[token("let")]
    Let,
    #[token(";")]
    #[display(fmt = ";")]
    Semicolon,
    #[token("=>")]
    #[display(fmt = "=>")]
    Arrow,
    #[token(",")]
    #[display(fmt = ",")]
    Comma,
    #[token(":")]
    #[display(fmt = ":")]
    Inherit,
    #[token("|")]
    #[display(fmt = "|")]
    Bar,
    // EXPRESSION KEYWORDS
    #[token("if")]
    If,
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
    // OPERATORS
    #[regex(
        r"\[|\]|\{|\}|\(|\)|\+|\-|\*|/|%|\*\*|==|!=|<|<=|>|>=|or|and|!|\.|=",
        lex_operator
    )]
    Operator(Operator),
    // LITERALS
    #[regex("true|false", lex_boolean)]
    Bool(bool),
    #[regex("Infinity|inf|NaN", lex_number)]
    #[regex("-?[0-9]*\\.?[0-9\\.]+", lex_number)]
    Number(f64),
    #[regex(r#""(\\"|[^"])*""#, lex_string)]
    String(&'t str),
    #[regex("[a-z_A-Z][a-z_A-Z0-9]*")]
    Identifier(&'t str),
    Eof,
    #[error]
    #[regex(r"[\n\f\r \t]+|([0-9]+[a-z_A-Z]+)|//.*", lex_error)]
    Error,
}

impl<'t> Token<'t> {
    pub(crate) fn is_stmt(&self) -> bool {
        use Token::*;

        matches!(self, Class | Function | Let)
    }

    pub(crate) fn is_expr(&self) -> bool {
        match self {
            Token::Operator(op) => !matches!(
                op,
                Operator::CurlyBracketClose
                    | Operator::SquareBracketClose
                    | Operator::RoundBracketClose
            ),
            Token::Identifier(_)
            | Token::String(_)
            | Token::Bool(_)
            | Token::Number(_)
            | Token::Break
            | Token::Continue
            | Token::For
            | Token::If
            | Token::Return
            | Token::While
            | Token::Super
            | Token::This
            | Token::Bar => true,
            _ => false,
        }
    }

    pub(crate) fn is_identifier(&self) -> bool {
        matches!(self, Token::Identifier(_))
    }
}

struct Source<'t> {
    inner: logos::Lexer<'t, Token<'t>>,
}

impl<'t> Source<'t> {
    pub fn new(input: &'t str) -> Self {
        Self {
            inner: Token::lexer(input),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct Lexeme<'t> {
    pub(crate) token: Token<'t>,
    pub(crate) slice: &'t str,
    pub(crate) span_start: usize,
    pub(crate) span_end: usize,
    pub(crate) intern_key: Option<Symbol>,
}

impl<'t> Lexeme<'t> {
    pub(crate) fn span(&self) -> Span {
        self.span_start..self.span_end
    }
}

impl<'t> Iterator for Source<'t> {
    type Item = Lexeme<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.inner.next()?;
        let slice = self.inner.slice();
        let span = self.inner.span();

        Some(Lexeme {
            token,
            slice,
            span_start: span.start,
            span_end: span.end,
            intern_key: None,
        })
    }
}

pub(crate) struct Lexer<'t> {
    // Logos lexer that lexes our source input
    inner: PeekNth<Source<'t>>,
    current_span: Option<Span>,
}

impl<'t> Lexer<'t> {
    pub(crate) fn new(input: &'t str) -> Self {
        Self {
            inner: peek_nth(Source::new(input)),
            current_span: None,
        }
    }

    pub(crate) fn span(&mut self) -> Option<Span> {
        self.inner.peek().map(|l| l.span())
    }

    pub(crate) fn slice(&mut self) -> Option<&str> {
        self.inner.peek().map(|l| l.slice)
    }

    pub(crate) fn peek(&mut self) -> Option<Lexeme> {
        self.peek_nth(0)
    }

    pub(crate) fn peek_nth(&mut self, nth: usize) -> Option<Lexeme> {
        self.inner.peek_nth(nth).copied()
    }

    pub(crate) fn current_span(&self) -> Span {
        self.current_span.clone().unwrap()
    }
}

impl<'t> Iterator for Lexer<'t> {
    type Item = Lexeme<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(lexeme) => {
                self.current_span = Some(lexeme.span());
                Some(lexeme)
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod test {
    use quickcheck_macros::quickcheck;

    use crate::{
        token::{operator::Operator, Lexeme, Lexer, Token},
        utils::test::lexer::{
            assert_empty, assert_error, assert_token, assert_tokens, first_token, op,
        },
    };

    #[test]
    fn lexer_peeks() {
        let mut lexer = Lexer::new("2 + 4");
        let two_l = Lexeme {
            slice: "2",
            token: Token::Number(2.0),
            span_start: 0,
            span_end: 1,
            intern_key: None,
        };

        assert_eq!(lexer.peek().unwrap(), two_l);
        // and it doesn't advance further
        assert_eq!(lexer.peek().unwrap(), two_l);
        // it can also peek nth lexeme

        assert_eq!(
            lexer.peek_nth(1).unwrap(),
            Lexeme {
                slice: "+",
                token: Token::Operator(Operator::Plus),
                span_start: 2,
                span_end: 3,
                intern_key: None
            }
        );
        let four_l = Lexeme {
            slice: "4",
            token: Token::Number(4.0),
            span_start: 4,
            span_end: 5,
            intern_key: None,
        };

        assert_eq!(lexer.peek_nth(2).unwrap(), four_l);
        // and it also doesn't advance the iterator
        assert_eq!(lexer.peek_nth(2).unwrap(), four_l);
        // we get None if we peek too far
        assert!(lexer.peek_nth(3).is_none());
    }

    #[test]
    fn lexer_returns_current_text_slice() {
        let mut lexer = Lexer::new("2 + 4");
        assert_eq!(lexer.slice(), Some("2"));
        // and it does so without advancing
        assert_eq!(lexer.slice(), Some("2"));
    }

    #[test]
    fn lexer_returns_current_span() {
        let mut lexer = Lexer::new("2 + 4");
        assert_eq!(lexer.span(), Some(0..1));
        // and it does so without advancing
        assert_eq!(lexer.span(), Some(0..1));
    }

    #[test]
    fn lexer_implements_iterator() {
        let mut lexer = Lexer::new("2 + 4");
        assert_eq!(
            lexer.next().unwrap(),
            Lexeme {
                token: Token::Number(2.0),
                slice: "2",
                span_start: 0,
                span_end: 1,
                intern_key: None
            }
        );
        assert_eq!(
            lexer.next().unwrap(),
            Lexeme {
                token: Token::Operator(Operator::Plus),
                slice: "+",
                span_start: 2,
                span_end: 3,
                intern_key: None
            }
        );
        assert_eq!(
            lexer.next().unwrap(),
            Lexeme {
                token: Token::Number(4.0),
                slice: "4",
                span_start: 4,
                span_end: 5,
                intern_key: None
            }
        );
    }

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
    fn lexer_tokenizes_nan() {
        let token = first_token("NaN");
        if let Token::Number(num) = token {
            assert!(num.is_nan());
        } else {
            panic!("Lexer didn't tokenize NaN");
        }
    }

    #[test]
    fn lexer_tokenizes_infinity() {
        assert_token("inf", Token::Number(f64::INFINITY));
        assert_token("Infinity", Token::Number(f64::INFINITY));
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
    fn lexer_tokenizes_keywords() {
        use Token::*;

        assert_token("fn", Function);
        assert_token("class", Class);
        assert_token("let", Let);
        assert_token("if", If);
        assert_token("else", Else);
        assert_token("while", While);
        assert_token("return", Return);
        assert_token("for", For);
        assert_token("break", Break);
        assert_token("continue", Continue);
        assert_token("this", This);
        assert_token("super", Super);
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

    #[test]
    fn lexer_skips_unnecessary_tokens() {
        // Skips comments
        assert_empty("//foobar");
        assert_tokens("while// smart comment", &[Token::While]);
        assert_empty("//while smart comment else if");
        // Skips newlines
        assert_empty("\x0A");
        assert_empty("\x0A\x0A\x0A\x0A");
        // Skips carriage return
        assert_empty("\x0D");
        assert_empty("\x0D\x0D\x0D\x0D");
        // Skips form feed
        assert_empty("\x0C");
        assert_empty("\x0C\x0C\x0C\x0C");
        // Spaces
        assert_empty(" ");
        assert_empty("       ");
    }

    #[test]
    fn lexer_tokenizes_binary_expression() {
        use self::Operator::*;
        use Token::*;

        assert_tokens(
            "foo == foo",
            &[Identifier("foo"), op(Compare), Identifier("foo")],
        );

        assert_tokens(
            "\"foo\" == \"foo\"",
            &[String("foo"), op(Compare), String("foo")],
        );

        assert_tokens("true and true", &[Bool(true), op(And), Bool(true)]);
        assert_tokens("false or true", &[Bool(false), op(Or), Bool(true)]);

        assert_tokens("0 + 1", &[Number(0.0), op(Plus), Number(1.0)]);
        assert_tokens("-0 + 1", &[Number(0.0), op(Plus), Number(1.0)]);
        assert_tokens("-0 + -1", &[Number(0.0), op(Plus), Number(-1.0)]);
    }

    #[test]
    fn lexer_reports_errors() {
        // Identifiers beginning with a number
        assert_error("123foo");
        assert_error("123foo");
    }

    #[test]
    fn lexer_tokenizes_misc_tokens() {
        assert_token(";", Token::Semicolon);
        assert_token(";;", Token::Semicolon);
        assert_token(";;;", Token::Semicolon);
        assert_token(",", Token::Comma);
        assert_token("=>", Token::Arrow);
        assert_token(":", Token::Inherit);
    }
}
