use crate::parse::Parser;

pub(crate) mod common;
pub(crate) mod parse;
pub(crate) mod token;

pub fn parse(code: &str) {
    let mut parser = Parser::new(code);
    parser.parse();
}

// pub fn parse_from_file<P: Into<Path>>(path: P) {}
