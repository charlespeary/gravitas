use analyzer::analyze;
use codespan_reporting::{
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use common::{CompilerDiagnostic, Symbols};
use parser::{
    parse,
    parse::{Ast, Program},
};
use std::path::Path;

pub(crate) fn log_errors(errors: Vec<impl CompilerDiagnostic>, symbols: Symbols, code: &str) {
    let mut files = SimpleFiles::new();
    let file_id = files.add("test.vt", code);
    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();
    let reader = symbols.into_reader();

    for err in errors {
        term::emit(
            &mut writer.lock(),
            &config,
            &files,
            &err.report(file_id, &reader),
        )
        .unwrap();
    }
}

pub(crate) fn compile(code: &str) -> Program {
    parse(code)
        .and_then(|(ast, symbols)| {
            if let Err(errors) = analyze(&ast) {
                return Err((errors, symbols));
            }
            Ok((ast, symbols))
        })
        .map_err(|(errors, symbols)| log_errors(errors, symbols, code))
        .expect("Compilation failed. See above errors to find out what went wrong.")
}

pub(crate) fn compile_file<P: AsRef<Path>>(path: P) {}
