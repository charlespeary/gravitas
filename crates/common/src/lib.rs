use codespan_reporting::diagnostic::Diagnostic;

pub trait CompilerDiagnostic: Sized {
    fn report(&self, file_id: usize) -> Diagnostic<usize>;
}

pub type Number = f64;
pub type Address = Number;

pub const MAIN_FUNCTION_NAME: &str = "main";
pub const LAMBDA_NAME: &str = "lambda";

pub type ProgramText = String;
