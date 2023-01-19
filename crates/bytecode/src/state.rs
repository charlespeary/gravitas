use std::collections::{HashMap, HashSet};

use common::{ProgramText, CLOSED_VARIABLE_PLACEHOLDER};

use crate::{callables::Class, MemoryAddress, Patch, Upvalue, Variable};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScopeType {
    Function,
    Block,
    Global,
    Class,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scope {
    pub scope_type: ScopeType,
    pub variables: Vec<Variable>,
    pub returned: bool,
    pub patches: HashSet<Patch>,
    pub starting_index: usize,
    pub upvalues: Vec<Upvalue>,
}

impl Scope {
    pub fn new(scope_type: ScopeType, starting_index: usize) -> Self {
        Self {
            scope_type,
            variables: vec![],
            patches: HashSet::new(),
            returned: false,
            starting_index,
            upvalues: vec![],
        }
    }

    pub fn make_enclosed_upvalue(&mut self, upvalue_index: usize) -> Upvalue {
        let upvalue = Upvalue {
            upvalue_index,
            is_local: false,
            local_index: 0,
        };

        self.upvalues.push(upvalue.clone());

        upvalue
    }

    pub fn close_variable(&mut self, index: usize) -> Upvalue {
        let mut var = self.variables.get_mut(index).unwrap();
        var.is_closed = true;

        let upvalues_len = self.upvalues.len();

        let upvalue_index = if upvalues_len == 0 {
            0
        } else {
            upvalues_len - 1
        };

        var.upvalue_index = Some(upvalue_index);

        let upvalue = Upvalue {
            local_index: var.index,
            upvalue_index,
            is_local: true,
        };

        self.upvalues.push(upvalue.clone());
        upvalue
    }
}

/// State of the generator
#[derive(Debug, Default, Clone)]
pub struct GeneratorState {
    pub scopes: Vec<Scope>,
    pub classes: HashMap<String, Class>,
}

fn search_var(scope: &Scope, name: &str) -> Option<(Variable, usize)> {
    for (index, var) in scope.variables.iter().enumerate() {
        if var.name == name {
            return Some((var.clone(), index));
        }
    }
    None
}

impl GeneratorState {
    pub fn new() -> Self {
        Self {
            // Initialize State with global scope
            scopes: vec![Scope::new(ScopeType::Global, 0)],
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
        let mut scopes = self
            .scopes
            .iter()
            .rev()
            .filter(|scope| scope.scope_type != ScopeType::Block);

        // If we are in a function that is inside in another function, then we are in a closure.
        let current_scope = scopes.next().map(|s| s.scope_type);
        let above_scope = scopes.next().map(|s| s.scope_type);

        match (current_scope, above_scope) {
            (Some(ScopeType::Function), Some(ScopeType::Function)) => true,
            _ => false,
        }
    }

    pub fn declared(&self) -> usize {
        self.current_scope().variables.len()
    }

    pub fn set_returned(&mut self) {
        self.current_scope_mut().returned = true;
    }

    pub fn did_return(&self) -> bool {
        self.current_scope().returned
    }

    pub fn enter_scope(&mut self, scope_type: ScopeType, starting_index: usize) {
        self.scopes.push(Scope::new(scope_type, starting_index))
    }

    pub fn leave_scope(&mut self) -> Scope {
        self.scopes.pop().expect("Tried to leave nest in top scope")
    }

    pub fn depth(&self) -> usize {
        // -1 because we don't count the local scope which is 0
        self.scopes
            .iter()
            .filter(|s| s.scope_type != ScopeType::Block)
            .count()
            - 1
    }

    pub fn declare_var(&mut self, name: ProgramText) {
        let depth = self.depth();
        // If we are in closure or function then offset equals to 0, otherwise we need to calculate blocks
        // above the current scope, because they don't reset the stack counter to
        // the beginning of the stack frame.
        let stack_offset: usize = if &self.current_scope().scope_type == &ScopeType::Function {
            0
        } else {
            self.scopes
                .iter()
                .rev()
                .skip(1)
                .take_while(|s| [ScopeType::Block, ScopeType::Global].contains(&s.scope_type))
                .map(|s| s.variables.len())
                .sum()
        };

        let scope = self.current_scope_mut();

        scope.variables.push(Variable {
            name: name.to_owned(),
            depth,
            index: stack_offset + scope.variables.len(),
            is_closed: false,
            upvalue_index: None,
        })
    }

    // This can't fail because it's either an upvalue or it's not defined and analyzer prevents the latter.
    pub fn search_upvalue_var(&mut self, name: &str) -> Upvalue {
        // let current_depth = self.depth();
        // // Offset to calculate if the scopes above are blocks, so they don't reset stack counter.
        // for scope in self.scopes.iter_mut().rev().skip(1) {
        //     println!("{}: SCOPE: {:?}", name, scope.declared);
        //     if let Some(var) = scope.declared.iter_mut().find(|var| var.name == name) {
        //         // println!("VAR.depth: {}, current_depth: {}", var.depth, current_depth);
        //         if var.depth != current_depth {
        //             var.closed = true;
        //             var.upvalue_index = Some(scope.upvalues_count);
        //             scope.upvalues_count += 1;
        //         }
        //         return Some(var);
        //     } else {
        //         println!("{}: NOT FOUND", name);
        //     }
        // }
        // None

        let scopes = self
            .scopes
            .iter_mut()
            .rev()
            // .skip(1)
            .filter(|scope| scope.scope_type != ScopeType::Block);

        let mut scopes_to_close: Vec<&mut Scope> = vec![];

        let mut upvalue = None;
        // mark it for closing
        for scope in scopes {
            if let Some((_, index)) = search_var(scope, name) {
                upvalue = Some(scope.close_variable(index));
                break;
            }

            scopes_to_close.push(scope);
        }

        let mut upvalue = upvalue.unwrap();

        // we skip the scope in which we found upvalue
        for scope in scopes_to_close.iter_mut().skip(1) {
            upvalue = scope.make_enclosed_upvalue(upvalue.upvalue_index);
        }

        return upvalue;
    }

    pub fn search_local_var(&self, name: &str) -> Option<Variable> {
        // there's always some scope
        let current_scope = self.scopes.last().unwrap();
        search_var(current_scope, name).map(|(var, _)| var)
    }

    pub fn find_var_address(&mut self, name: &str) -> MemoryAddress {
        if let Some(local_variable) = self.search_local_var(name) {
            return MemoryAddress::Local(local_variable.index);
        }

        let upvalue_index = self.search_upvalue_var(name).upvalue_index;
        MemoryAddress::Upvalue(upvalue_index)

        // let in_closure = self.is_in_closure();
        // let var = self
        //     .search_var(name)
        //     .map(|var| {
        //         if var.closed && in_closure {
        //             MemoryAddress::Upvalue(var.upvalue_index.unwrap())
        //         } else {
        //             MemoryAddress::Local(var.index)
        //         }
        //     })
        //     // TODO: Deal with STD lib
        //     .unwrap();

        // var
    }

    // pub fn scope_closed_variables(&self) -> Vec<&Variable> {
    //     self.current_scope()
    //         .variables
    //         .iter()
    //         .filter(|v| v.is_closed)
    //         .collect()
    // }

    pub fn scope_upvalues(&self) -> Vec<&Upvalue> {
        self.current_scope().upvalues.iter().collect()
    }

    pub(crate) fn add_patch(&mut self, patch: Patch) {
        self.current_scope_mut().patches.insert(patch);
    }

    pub(crate) fn remove_patch(&mut self, patch: &Patch) {
        self.current_scope_mut().patches.remove(patch);
    }
}
