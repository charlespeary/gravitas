use std::str::FromStr;

use codespan_reporting::diagnostic::Diagnostic;

pub trait CompilerDiagnostic: Sized {
    fn report(&self, file_id: usize) -> Diagnostic<usize>;
}

pub type Number = f64;
pub type Address = Number;

pub const MAIN_FUNCTION_NAME: &str = "main";
pub const LAMBDA_NAME: &str = "lambda";
pub type ProgramText = String;

// STD function names

#[derive(Hash, PartialEq, PartialOrd, Eq, Clone, Debug)]
pub enum BuiltInFunction {
    Clock,
    Print,
}

impl Into<String> for BuiltInFunction {
    fn into(self) -> String {
        match self {
            BuiltInFunction::Clock => "clock".to_string(),
            BuiltInFunction::Print => "print".to_string(),
        }
    }
}

impl FromStr for BuiltInFunction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "clock" => BuiltInFunction::Clock,
            "print" => BuiltInFunction::Print,
            _ => return Err(()),
        })
    }
}

pub fn find_std_function(name: &str) -> Option<BuiltInFunction> {
    BuiltInFunction::from_str(name).ok()
}
