use codespan_reporting::diagnostic::Diagnostic;

pub trait CompilerDiagnostic: Sized {
    fn report(&self, file_id: usize) -> Diagnostic<usize>;
}

pub type Number = f64;
pub type Address = Number;

pub const MAIN_FUNCTION_NAME: &str = "main";
pub const LAMBDA_NAME: &str = "lambda";
pub const CONSTRUCTOR_NAME: &str = "constructor";
pub type ProgramText = String;
