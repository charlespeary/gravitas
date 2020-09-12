use std::collections::HashMap;

use anyhow::{anyhow, Result};
use derive_more::Display;
use logos::Logos;

use crate::hashmap;

#[derive(Logos, Debug, Display, Clone, PartialEq)]
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
    BangEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,
    #[token("==")]
    Equal,
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
    Number(f64),
    #[regex("\"[a-zA-Z ]+\"", | lex | lex.slice().parse())]
    Text(String),
    #[regex("[a-zA-Z]+", | lex | lex.slice().parse())]
    Identifier(String),
    #[error]
    // We can also use this variant to define whitespace,
    // or any other matches we wish to skip.
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

#[derive(Display)]
pub enum Affix {
    Infix,
    Prefix,
}

impl Token {
    /// Get binding power of the token
    /// e.g infix minus should have smaller binding power
    /// than infix star, so we can parse the expressions in correct order.
    /// ```
    ///         2 + 2 * 8
    ///     should be parsed into   
    /// Expr::Binary(2, + , Expr::Binary(2 * 8))
    /// ```
    pub fn bp(&self, affix: Affix) -> Result<usize> {
        let error = || {
            Err(anyhow!(
                "No rule specified for token {} as an {}",
                self,
                affix
            ))
        };

        Ok(match affix {
            Affix::Prefix => match self {
                Token::Minus => 7,
                Token::Bang => 7,
                Token::OpenParenthesis => 0,
                _ => return error(),
            },
            Affix::Infix => match self {
                Token::BangEqual => 4,
                Token::Equal => 4,
                Token::Greater => 4,
                Token::GreaterEqual => 4,
                Token::Less => 4,
                Token::LessEqual => 4,
                Token::Plus => 5,
                Token::Minus => 5,
                Token::Star => 6,
                Token::Divide => 6,
                Token::CloseParenthesis => 0,
                _ => return error(),
            },
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Token is able to find a rule for corresponding kind of affix.
    #[test]
    fn finds_rule() {
        assert_eq!(Token::Minus.bp(Affix::Infix).expect("Rule not found"), 5);

        assert_eq!(Token::Minus.bp(Affix::Prefix).expect("Rule not found"), 7);
    }

    /// It throws an error if it doesn't find a corresponding rule.
    #[test]
    #[should_panic(expected = "Rule not found")]
    fn rule_not_found() {
        Token::Error.bp(Affix::Infix).expect("Rule not found");
    }
}
