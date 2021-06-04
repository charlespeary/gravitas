use crate::parse::Parser;
pub use crate::parse::Program;
use crate::utils::error::ParseError;
use std::{fs, path::Path};

pub(crate) mod parse;
pub(crate) mod token;
pub mod utils;

pub type ParseResult = Result<Program, Vec<ParseError>>;

pub fn parse(code: &str) -> ParseResult {
    let parser = Parser::new(code);
    parser.parse()
}

pub fn parse_file<P: AsRef<Path>>(path: P) -> ParseResult {
    let code = fs::read_to_string(path).unwrap();
    parse(&code)
}
