use anyhow::{Context, Result};
pub use enum_as_inner::EnumAsInner;

use crate::bytecode::stmt::var::Variable;
use crate::{bytecode::Address, std::GLOBALS};

#[derive(EnumAsInner, Debug, Clone, Copy, PartialEq)]
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
}

impl Scope {
    pub fn new(scope_type: ScopeType) -> Self {
        Self {
            scope_type,
            declared: vec![],
            closed: vec![],
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

    pub fn declare_var(&mut self, name: &str) {
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

    pub fn find_var(&mut self, name: &str) -> Result<Address> {
        let current_depth = self.depth();
        let in_closure = self.is_in_closure();
        self.search_var(name)
            .map(|var| {
                if var.closed && in_closure {
                    Address::Upvalue(var.index, current_depth - var.depth)
                } else {
                    Address::Local(var.index)
                }
            })
            .or_else(|| GLOBALS.get(name).map(|_| Address::Global(name.to_owned())))
            .with_context(|| format!("{} doesn't exist", name))
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
}

#[cfg(test)]
mod test {
    use super::*;

    // GeneratorState is initialized with one global scope
    #[test]
    fn new() {
        let state = GeneratorState::new();
        assert_eq!(state.scopes.len(), 1);
        assert_eq!(state.scopes[0], Scope::new(ScopeType::Global))
    }

    // Generate GeneratorState populated with given scope type
    fn state_with_scope(scope_type: ScopeType) -> GeneratorState {
        let mut state = GeneratorState::new();
        state.enter_scope(scope_type);
        state
    }

    // GeneratorState should return last scope in the scopes vector, mutable or not, depending on the called method.
    #[test]
    fn current_scope() {
        let mut state = state_with_scope(ScopeType::Block);
        assert_eq!(state.current_scope(), &Scope::new(ScopeType::Block));
        assert_eq!(state.current_scope_mut(), &mut Scope::new(ScopeType::Block));
    }

    // GeneratorState should return true if the outermost scope is a closure. Otherwise it should return false.
    #[test]
    fn is_in_closure() {
        use ScopeType::*;

        let state = state_with_scope(Closure);
        assert!(state.is_in_closure());

        for invalid_scope in [Function, Global, Block].iter() {
            let state = state_with_scope(*invalid_scope);
            assert!(!state.is_in_closure());
        }
    }

    // GeneratorState should return correct number of declared variables within scope
    #[test]
    fn declared() {
        let mut state = GeneratorState::new();
        // By default we have only the global scope without any variables
        assert_eq!(state.declared(), 0);
        // We declare a variable, so the counter goes up.
        state.declare_var("foo");
        assert_eq!(state.declared(), 1);
        // We enter new scope, so it doesn't have any declared variables
        state.enter_scope(ScopeType::Closure);
        assert_eq!(state.declared(), 0);
        // And now it does
        state.declare_var("bar");
        assert_eq!(state.declared(), 1);
    }

    // GeneratorState correctly flags if current scope returned and also returns this information correctly.
    #[test]
    fn returns() {
        let mut state = GeneratorState::new();
        // Empty scope couldn't return
        assert!(!state.did_return());
        // We set it and now it's true
        state.set_returned();
        assert!(state.did_return());
        // But now as we enter a new scope it goes back to being false
        state.enter_scope(ScopeType::Closure);
        assert!(!state.did_return());
    }

    // GeneratorState should be able to correctly enter and leave new scope.
    #[test]
    fn enter_and_leave_scope() {
        let mut state = GeneratorState::new();
        // We start with a global scope
        assert_eq!(state.current_scope(), &Scope::new(ScopeType::Global));
        // As we enter new scope the type of scope goes to closure
        state.enter_scope(ScopeType::Closure);
        assert_eq!(state.current_scope(), &Scope::new(ScopeType::Closure));
        // And now as we leave it goes back to a global one
        state.leave_scope();
        assert_eq!(state.current_scope(), &Scope::new(ScopeType::Global));
    }

    // GeneratorState should return correct depth level.
    // Disclaimer: block scopes are discarded in this calculation as they do not change
    // the call stack and because of that stack start stays the same.
    #[test]
    fn depth_level() {
        let mut state = GeneratorState::new();
        // We are one level deep with global scope.
        assert_eq!(state.depth(), 1);
        // As we enter function the depth level increments
        state.enter_scope(ScopeType::Function);
        assert_eq!(state.depth(), 2);
        // But it doesn't change as we enter a new block
        state.enter_scope(ScopeType::Block);
        assert_eq!(state.depth(), 2);
        // It also increments when we enter a new closure
        state.enter_scope(ScopeType::Closure);
        assert_eq!(state.depth(), 3);
        // And it decrements whenever we leave the scope
        state.leave_scope();
        assert_eq!(state.depth(), 2);
        // It's gonna be block, so it won't change
        state.leave_scope();
        assert_eq!(state.depth(), 2);
        // And this one is a function, so it goes down
        state.leave_scope();
        assert_eq!(state.depth(), 1);
    }

    // GeneratorState should be able to declare variables and then find them.
    // Declared variables should be instantiated with correct depth based on type of the current scope.
    #[test]
    fn declare_and_search_var() {
        let mut state = GeneratorState::new();

        assert_eq!(state.declared(), 0);
        state.declare_var("foo");
        assert_eq!(state.declared(), 1);

        // After we declare variable we should be able to find it
        // It should have index 0, because there are no other variables defined
        // and depth equal to 1 which corresponds to only one scope we're currently in
        assert_eq!(
            state.search_var("foo").unwrap(),
            &Variable {
                name: String::from("foo"),
                depth: 1,
                index: 0,
                closed: false,
            }
        );

        state.declare_var("bar");
        assert_eq!(state.declared(), 2);

        // After we declare another variable the depth stays the same, but index increments
        assert_eq!(
            state.search_var("bar").unwrap(),
            &Variable {
                name: String::from("bar"),
                depth: 1,
                index: 1,
                closed: false,
            }
        );

        state.enter_scope(ScopeType::Block);
        state.declare_var("zoo");

        // Block doesn't change the depth level
        assert_eq!(
            state.search_var("zoo").unwrap(),
            &Variable {
                name: String::from("zoo"),
                depth: 1,
                index: 2,
                closed: false,
            }
        );

        state.enter_scope(ScopeType::Function);
        // New scope doesn't have any variables yet
        assert_eq!(state.declared(), 0);
        state.declare_var("doo");
        assert_eq!(state.declared(), 1);
        assert_eq!(
            state.search_var("doo").unwrap(),
            &Variable {
                name: String::from("doo"),
                depth: 2,
                index: 0,
                closed: false,
            }
        );

        // We are also able to shadow variables in the upper scope
        state.declare_var("foo");
        assert_eq!(
            state.search_var("foo").unwrap(),
            &Variable {
                name: String::from("foo"),
                depth: 2,
                index: 1,
                closed: false,
            }
        )
    }

    // GeneratorState should return address to a variable based on its name.
    // It should otherwise return an error.
    #[test]
    fn find_var() {
        let mut state = GeneratorState::new();
        // Handle locals.
        state.declare_var("foo");
        // Since it's firstly declared variable in the current scope, then it should be at local address 0.
        assert_eq!(state.find_var("foo").unwrap(), Address::Local(0));

        // The next one is gonna be one index above.
        state.declare_var("bar");
        assert_eq!(state.find_var("bar").unwrap(), Address::Local(1));

        // Handle upvalues.
        // Whenever we enter closure and we try to find variable that is
        // one level of nesting above, then the address will turn into Upvalue
        // that is also capable of pointing to closed values in the closure's special environment.
        state.enter_scope(ScopeType::Closure);
        assert_eq!(state.find_var("foo").unwrap(), Address::Upvalue(0, 1));
        // Handle globals.
        // For now the functions from standard library are global by default, so e.g
        // when we try to find "print" variable we will find the globally defined function "print"
        // from standard library.
        assert_eq!(
            state.find_var("print").unwrap(),
            Address::Global(String::from("print"))
        );

        // Handle errors.
        // It should return an error if it doesn't exist.
        assert!(state.find_var("nonexistentone").is_err())
    }

    // GeneratorState should return reference to the variables in current scope.
    #[test]
    fn scope_variables() {
        let mut state = GeneratorState::new();
        assert_eq!(state.scope_variables(), &vec![]);
        state.declare_var("foo");
        assert_eq!(
            state.scope_variables(),
            &vec![Variable {
                name: String::from("foo"),
                index: 0,
                depth: 1,
                closed: false,
            }]
        );
        state.enter_scope(ScopeType::Closure);
        assert_eq!(state.scope_variables(), &vec![])
    }
}
