use analyzer::analyze;
use bytecode::generate_bytecode;
use codespan_reporting::{
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use common::CompilerDiagnostic;
use parser::{parse, parse::Program};
use std::path::Path;
use vm::{run, runtime_value::RuntimeValue};

pub(crate) fn log_errors(errors: Vec<impl CompilerDiagnostic>, code: &str) {
    let mut files = SimpleFiles::new();
    let file_id = files.add("test.vt", code);
    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();

    for err in errors {
        term::emit(&mut writer.lock(), &config, &files, &err.report(file_id)).unwrap();
    }
}

pub(crate) fn compile(code: &str) -> Program {
    parse(code)
        .and_then(|ast| {
            if let Err(errors) = analyze(&ast) {
                return Err(errors);
            }
            Ok(ast)
        })
        .map_err(|errors| log_errors(errors, code))
        .expect("Compilation failed. See above errors to find out what went wrong.")
}

pub(crate) fn compile_and_run(code: &str, debug: bool) -> RuntimeValue {
    let ast = compile(code);

    analyze(&ast)
        .map_err(|errors| log_errors(errors, &code))
        .expect("Static analysis failed. Investigate above errors to find the cause.");

    let bytecode = generate_bytecode(ast.clone())
        .map_err(|error| println!("TODO: generation errors"))
        .expect("Bytecode generation failed. Investigate above errors to find the cause.");

    run(bytecode, debug)
}

pub(crate) fn compile_file<P: AsRef<Path>>(path: P) {}
