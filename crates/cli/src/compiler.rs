use analyzer::analyze;
use codespan_reporting::{
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use common::CompilerDiagnostic;
use lasso::Rodeo;
use parser::{
    parse,
    parse::{Program, ProgramErrors},
};
use std::path::Path;

fn show_errors(errors: Vec<impl CompilerDiagnostic>, symbols: Rodeo, code: &str) {
    let mut files = SimpleFiles::new();
    let file_id = files.add("test.vt", code);
    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();

    for err in errors {
        term::emit(
            &mut writer.lock(),
            &config,
            &files,
            &err.report(file_id, &symbols),
        )
        .unwrap();
    }
}

pub(crate) fn compile(code: &str) {
    let x = parse(code)
        .and_then(analyze)
        .map_err(|(errors, symbols)| show_errors(errors, symbols, code));
}

pub(crate) fn compile_file<P: AsRef<Path>>(path: P) {}
