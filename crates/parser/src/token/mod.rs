use logos::Logos;

use operator::{lex_operator, Operator};

pub(crate) mod operator;

#[derive(Logos, Debug, PartialEq)]
pub(crate) enum Token {
    // KEYWORDS
    #[regex(r"\+|\-|\*|/|%|\*\*|==|!=|<|<=|>|>=|or|and|!|\.|=", lex_operator)]
    Operator(Operator),
    // LITERALS
    #[regex("-?[0-9]*\\.?[0-9]+", | lex | lex.slice().parse())]
    Number(f64),
    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

#[cfg(test)]
mod test {}
