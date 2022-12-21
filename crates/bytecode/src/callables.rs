use std::fmt::Display;

use common::ProgramText;
use prettytable::Table;

use crate::chunk::{self, chunk_into_rows, Chunk, Constant};

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: ProgramText,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Class {
    pub name: ProgramText,
    pub constructor: Function,
    pub methods: Vec<Function>,
    pub super_class: Option<Box<Class>>,
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();

        table.add_row(row!["Name", "Arity"]);
        table.add_row(row![self.name, self.arity]);

        for row in chunk_into_rows(self.chunk.clone()) {
            table.add_row(row);
        }

        table.fmt(f)?;

        fn extract_functions(chunk: Chunk) -> Vec<Function> {
            fn extractor(chunk: Chunk, functions: &mut Vec<Function>) {
                for constant in chunk.constants {
                    match constant {
                        Constant::Function(fun) => {
                            functions.push(fun.clone());
                            extractor(fun.chunk, functions);
                        }
                        _ => {}
                    }
                }
            }
            let mut functions = vec![];
            extractor(chunk, &mut functions);
            functions
        }

        let functions_inside_chunk = extract_functions(self.chunk.clone());

        for function in functions_inside_chunk {
            write!(f, "{}", function)?;
        }

        Ok(())
    }
}
