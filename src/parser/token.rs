use derive_more::Display;
use enum_as_inner::EnumAsInner;
use logos::{Lexer, Logos};

use crate::parser::operator::{lex_operator, Operator};

fn lex_text(lex: &mut Lexer<Token>) -> Option<String> {
    let slice: String = lex.slice().parse().ok()?;
    Some(slice[1..slice.len() - 1].to_owned())
}

#[derive(Logos, Debug, Display, Clone, PartialEq, EnumAsInner)]
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
    #[regex(r"\+|\-|\*|/|>|>=|<=|<|!|!=|=|==|\\.", lex_operator)]
    Operator(Operator),
    #[token(";")]
    Semicolon,
    // #[regex("\/\/.*", logos::skip)]
    // Comment,
    #[token("if")]
    If,
    #[token("elif")]
    ElseIf,
    #[token("else")]
    Else,
    #[token("match")]
    Match,
    #[token("_")]
    Default,
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
    #[token("=>")]
    Arrow,
    #[regex("-?[0-9]*\\.?[0-9]+", | lex | lex.slice().parse())]
    Number(f64),
    #[regex("\"[^\"]*\"", lex_text)]
    Text(String),
    #[regex("[a-zA-Z]+", | lex | lex.slice().parse())]
    Identifier(String),
    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

impl Token {
    /// Helper to determine whether token is associated with parsing the statements
    pub fn is_stmt(&self) -> bool {
        matches!(self, Token::Var | Token::Class | Token::Function)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Checks whether Token is of type related to statements. E.g "var" keyword
    #[test]
    fn is_stmt() {
        assert!(Token::Var.is_stmt());
        assert!(Token::Function.is_stmt());
        assert!(Token::Class.is_stmt());
    }

    #[test]
    fn isnt_stmt() {
        assert!(!Token::Bar.is_stmt());
        assert!(!Token::OpenParenthesis.is_stmt());
        assert!(!Token::CloseParenthesis.is_stmt());
        assert!(!Token::OpenBrace.is_stmt());
        assert!(!Token::CloseBrace.is_stmt());
        assert!(!Token::Coma.is_stmt());
        assert!(!Token::Semicolon.is_stmt());
        assert!(!Token::If.is_stmt());
        assert!(!Token::ElseIf.is_stmt());
        assert!(!Token::Else.is_stmt());
        assert!(!Token::Match.is_stmt());
        assert!(!Token::Default.is_stmt());
        assert!(!Token::False.is_stmt());
        assert!(!Token::True.is_stmt());
        assert!(!Token::While.is_stmt());
        assert!(!Token::For.is_stmt());
        assert!(!Token::And.is_stmt());
        assert!(!Token::Or.is_stmt());
        assert!(!Token::Break.is_stmt());
        assert!(!Token::Continue.is_stmt());
        assert!(!Token::Return.is_stmt());
        assert!(!Token::Super.is_stmt());
        assert!(!Token::This.is_stmt());
        assert!(!Token::Null.is_stmt());
        assert!(!Token::Arrow.is_stmt());
        assert!(!Token::Number(0.0).is_stmt());
        assert!(!Token::Text(String::from("test")).is_stmt());
        assert!(!Token::Identifier(String::from("test")).is_stmt());
        assert!(!Token::Error.is_stmt());
    }
}
