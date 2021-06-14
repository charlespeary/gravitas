use common::Symbol;

use crate::Analyzer;

pub(crate) struct Function {
    pub(crate) arity: usize,
}

impl Analyzer {
    pub(crate) fn declare_function(&mut self, name: Symbol, arity: usize) {
        self.functions.insert(name, Function { arity });
    }
}
