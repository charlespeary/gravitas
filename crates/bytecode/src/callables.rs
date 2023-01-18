use std::fmt::Display;

use common::ProgramText;
use prettytable::Table;

use crate::{
    chunk::{chunk_into_rows, Chunk},
    stmt::GlobalPointer,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: ProgramText,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Class {
    pub name: ProgramText,
    pub methods: Vec<GlobalPointer>,
    pub super_class: Option<Box<GlobalPointer>>,
    pub constructor: Option<GlobalPointer>,
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

        Ok(())
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();

        table.add_row(row!["Name", "Number of methods"]);
        table.add_row(row![self.name, self.methods.len()]);
        table.fmt(f)?;

        for method in &self.methods {
            write!(f, "{}", method)?;
        }

        Ok(())
    }
}
