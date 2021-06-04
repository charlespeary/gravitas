use codespan_reporting::diagnostic::Diagnostic;

pub trait CompilerDiagnostic {
    fn report(&self, file_id: usize) -> Diagnostic<usize>;
}
