use codespan_reporting::diagnostic::Diagnostic;
use lasso::{Rodeo, RodeoReader, Spur};

pub trait CompilerDiagnostic: Sized {
    fn report(&self, file_id: usize, reader: &SymbolsReader) -> Diagnostic<usize>;
}

pub type Number = f64;
pub type Address = Number;
pub type Symbol = Spur;
pub type Symbols = Rodeo;
pub type SymbolsReader = RodeoReader;
