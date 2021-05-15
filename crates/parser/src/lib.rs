use crate::parse::{Parser, ParserOutput};
use std::{fs, path::Path};

pub(crate) mod common;
pub(crate) mod parse;
pub(crate) mod token;

pub fn parse(code: &str) -> ParserOutput {
    let parser = Parser::new(code);
    parser.parse()
}

pub fn parse_file<P: AsRef<Path>>(path: P) -> ParserOutput {
    let code = fs::read_to_string(path).unwrap();
    parse(&code)
}
