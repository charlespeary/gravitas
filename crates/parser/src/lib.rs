use crate::parse::{Parser, ParserOutput, Program};
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    term,
};
use std::{fs, path::Path};

pub(crate) mod common;
pub(crate) mod parse;
pub(crate) mod token;

pub type ParseResult = Result<Program, ()>;

pub fn parse(code: &str) -> ParseResult {
    let parser = Parser::new(code);
    match parser.parse() {
        Ok(program) => Ok(program),
        Err(errors) => {
            let mut files = SimpleFiles::new();
            let file_id = files.add("test.vt", code);
            let writer = StandardStream::stderr(ColorChoice::Always);
            let config = codespan_reporting::term::Config::default();

            for err in errors {
                term::emit(&mut writer.lock(), &config, &files, &err.report(file_id)).unwrap();
            }

            Err(())
        }
    }
}

pub fn parse_file<P: AsRef<Path>>(path: P) -> ParseResult {
    let code = fs::read_to_string(path).unwrap();
    parse(&code)
}
