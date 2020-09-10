use std::collections::VecDeque;
use std::iter::Fuse;
use std::marker::PhantomData;

use logos::Logos;
use ordered_float::NotNan;

#[derive(Logos, Debug, Clone, Hash, Eq, PartialEq)]
pub enum Token {
    #[token("|")]
    Bar,
    #[token("(")]
    OpenParenthesis,
    #[token(")")]
    CloseParenthesis,
    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,
    #[token(",")]
    Coma,
    #[token(".")]
    Dot,
    #[token("-")]
    Minus,
    #[token("+")]
    Plus,
    #[token("*")]
    Star,
    #[token("/")]
    Divide,
    #[token("%")]
    Modulo,
    #[token(";")]
    Semicolon,
    #[token("!")]
    Bang,
    #[token("!=")]
    BangEquals,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEquals,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEquals,
    #[token("==")]
    Compare,
    #[token("=")]
    Assign,
    #[token("//")]
    Comment,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("false")]
    False,
    #[token("true")]
    True,
    #[token("var")]
    Var,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("fn")]
    Function,
    #[token("return")]
    Return,
    #[token("class")]
    Class,
    #[token("super")]
    Super,
    #[token("this")]
    This,
    #[token("null")]
    Null,
    #[token("print")]
    Print,
    #[token("=>")]
    Arrow,
    #[regex("-?[0-9]*\\.?[0-9]+", | lex | lex.slice().parse())]
    Number(NotNan<f64>),
    #[regex("\"[a-zA-Z]+\"", | lex | lex.slice().parse())]
    Text(String),
    #[regex("[a-zA-Z]+", | lex | lex.slice().parse())]
    Identifier(String),
    #[error]
    // We can also use this variant to define whitespace,
    // or any other matches we wish to skip.
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

impl Token {
    pub fn bp(&self) -> usize {
        match self {
            Token::Plus => 5,
            Token::Star => 6,
            Token::Divide => 6,
            _ => 0,
        }
    }
}
