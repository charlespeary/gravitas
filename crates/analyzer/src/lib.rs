use parser::utils::error::ParseError;
use parser::Program;

pub(crate) mod error;

pub type AnalyzerResult = Result<Program, Vec<ParseError>>;

pub struct Analyzer {
    program: Program,
}

impl Analyzer {
    pub fn new(program: Program) -> Self {
        Self { program }
    }

    pub fn analyze(mut self) -> AnalyzerResult {
        Ok(self.program)
    }
}

pub fn analyze(program: Program) -> AnalyzerResult {
    let mut analyzer = Analyzer::new(program);
    analyzer.analyze()
}
