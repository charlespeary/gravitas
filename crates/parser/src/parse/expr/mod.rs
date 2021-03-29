use crate::parse::{expr::atom::Atom, ParseResult, Parser};

pub(crate) mod atom;

pub(crate) enum Expression {
    Atom(Atom),
}

macro_rules! impl_into_expression {
    ($($variant:ident), *) => {
        $(impl Into<Expression> for $variant {
            fn into(self) -> Expression {
                Expression::$variant(self)
            }
        })*
    };
}

impl_into_expression!(Atom);

macro_rules! try_expr {
    ($val: expr) => {
        Into::<Expression>::into($val?)
    };
}

impl<'a> Parser<'a> {
    pub(super) fn parse_expression(&mut self) -> ParseResult<Expression> {
        Ok(match self.peek()? {
            _ => try_expr!(self.parse_atom()),
        })
    }
}
