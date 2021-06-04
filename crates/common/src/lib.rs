use codespan_reporting::diagnostic::Diagnostic;
use lasso::{Rodeo, Spur};

pub trait CompilerDiagnostic: Sized {
    fn report(&self, file_id: usize, symbols: &Symbols) -> Diagnostic<usize>;
}

pub type Symbol = Spur;
pub type Symbols = Rodeo;
