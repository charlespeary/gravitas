pub use enum_as_inner::EnumAsInner;

use crate::bytecode::stmt::var::{Upvalue, Variable};

#[derive(EnumAsInner, Debug, Clone)]
pub enum ScopeType {
    Function,
    Closure,
    Block,
    Global,
}

#[derive(Debug, Clone)]
pub struct Scope {
    pub scope_type: ScopeType,
    pub declared: Vec<Variable>,
    pub returned: bool,
}

impl Scope {
    pub fn new(scope_type: ScopeType) -> Self {
        Self {
            scope_type,
            declared: vec![],
            returned: false,
        }
    }
}

/// State of the generator
#[derive(Debug, Default, Clone)]
pub struct GeneratorState {
    pub scopes: Vec<Scope>,
}

impl GeneratorState {
    pub fn new() -> Self {
        Self {
            // Initialize State with global scope
            scopes: vec![Scope::new(ScopeType::Global)],
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
        self.scopes
            .last()
            .map_or(false, |n| n.scope_type.as_closure().is_some())
    }

    pub fn is_in_function(&self) -> bool {
        self.scopes
            .last()
            .map_or(false, |n| n.scope_type.as_function().is_some())
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

    pub fn is_in_global_scope(&self) -> bool {
        self.scopes.len() == 1
    }

    pub fn depth(&self) -> usize {
        self.scopes.len()
    }

    pub fn declare_var(&mut self, name: &str) {
        let depth = self.depth();
        let scope = self.current_scope_mut();
        scope.declared.push(Variable {
            name: name.to_owned(),
            depth,
            index: scope.declared.len(),
            closed: false,
        })
    }

    pub fn find_var(&mut self, name: &str) -> Option<&Variable> {
        let current_depth = self.depth();
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

    pub fn scope_variables(&self) -> &Vec<Variable> {
        &self.current_scope().declared
    }
}
