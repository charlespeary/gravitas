use analyzer::analyze;
use codespan_reporting::{
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use common::CompilerDiagnostic;
use parser::parse;
use std::path::Path;

pub(crate) fn compile(code: &str) {
    parse(code)
        .map(analyze)
        .map_err(|errors| {
            let mut files = SimpleFiles::new();
            let file_id = files.add("test.vt", code);
            let writer = StandardStream::stderr(ColorChoice::Always);
            let config = codespan_reporting::term::Config::default();

            for err in errors {
                term::emit(&mut writer.lock(), &config, &files, &err.report(file_id)).unwrap();
            }
        })
        .unwrap()
        .unwrap();
}

pub(crate) fn compile_file<P: AsRef<Path>>(path: P) {}
