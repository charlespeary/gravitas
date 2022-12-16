use std::collections::HashMap;

use common::ProgramText;

use crate::{callables::Class, MemoryAddress, Patch, Variable};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScopeType {
    Function,
    Closure,
    Block,
    Global,
    Class,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub scope_type: ScopeType,
    pub declared: Vec<Variable>,
    pub closed: Vec<Variable>,
    pub returned: bool,
    pub patches: Vec<Patch>,
}

impl Scope {
    pub fn new(scope_type: ScopeType) -> Self {
        Self {
            scope_type,
            declared: vec![],
            closed: vec![],
            patches: vec![],
            returned: false,
        }
    }
}

/// State of the generator
#[derive(Debug, Default, Clone)]
pub struct GeneratorState {
    pub scopes: Vec<Scope>,
    pub classes: HashMap<String, Class>,
}

impl GeneratorState {
    pub fn new() -> Self {
        Self {
            // Initialize State with global scope
            scopes: vec![Scope::new(ScopeType::Global)],
            ..Default::default()
        }
    }

    pub fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes
            .last_mut()
            .expect("Tried to access scope above the global one.")
    }

    pub fn current_scope(&self) -> &Scope {
        self.scopes
            .last()
            .expect("Tried to access scope above the global one.")
    }

    pub fn is_in_closure(&self) -> bool {
        self.scopes.last().map_or(false, |scope| {
            if let ScopeType::Closure = scope.scope_type {
                true
            } else {
                false
            }
        })
    }

    pub fn declared(&self) -> usize {
        self.current_scope().declared.len()
    }

    pub fn set_returned(&mut self) {
        self.current_scope_mut().returned = true;
    }

    pub fn did_return(&self) -> bool {
        self.current_scope().returned
    }

    pub fn enter_scope(&mut self, scope_type: ScopeType) {
        self.scopes.push(Scope::new(scope_type))
    }

    pub fn leave_scope(&mut self) -> Scope {
        self.scopes.pop().expect("Tried to leave nest in top scope")
    }

    pub fn depth(&self) -> usize {
        self.scopes
            .iter()
            .filter(|s| s.scope_type != ScopeType::Block)
            .count()
    }

    pub fn declare_var(&mut self, name: ProgramText) {
        let depth = self.depth();
        // If we are in closure or function then offset equals to 0, otherwise we need to calculate blocks
        // above the current scope, because they don't reset the stack counter to
        // the beginning of the stack frame.
        let stack_offset: usize = if [ScopeType::Function, ScopeType::Closure]
            .contains(&self.current_scope().scope_type)
        {
            0
        } else {
            self.scopes
                .iter()
                .rev()
                .skip(1)
                .take_while(|s| [ScopeType::Block, ScopeType::Global].contains(&s.scope_type))
                .map(|s| s.declared.len())
                .sum()
        };

        let scope = self.current_scope_mut();

        scope.declared.push(Variable {
            name: name.to_owned(),
            depth,
            index: stack_offset + scope.declared.len(),
            closed: false,
        })
    }

    pub fn search_var(&mut self, name: &str) -> Option<&Variable> {
        let current_depth = self.depth();
        // Offset to calculate if the scopes above are blocks, so they don't reset stack counter.
        for scope in self.scopes.iter_mut().rev() {
            if let Some(var) = scope.declared.iter_mut().find(|var| var.name == name) {
                if var.depth != current_depth {
                    var.closed = true;
                }
                return Some(var);
            }
        }
        None
    }

    pub fn find_var(&mut self, name: &str) -> MemoryAddress {
        let current_depth = self.depth();
        let in_closure = self.is_in_closure();
        self.search_var(name)
            .map(|var| {
                if var.closed && in_closure {
                    MemoryAddress::Upvalue(var.index, current_depth - var.depth)
                } else {
                    MemoryAddress::Local(var.index)
                }
            })
            .expect("Generator is in a bad state. Analyzer didn't ensure that variable is defined.")
    }

    pub fn scope_variables(&self) -> &Vec<Variable> {
        &self.current_scope().declared
    }

    pub fn scope_closed_variables(&self) -> Vec<&Variable> {
        self.current_scope()
            .declared
            .iter()
            .filter(|v| v.closed)
            .collect()
    }

    pub(crate) fn add_patch(&mut self, patch: Patch) {
        self.current_scope_mut().patches.push(patch);
    }
}
