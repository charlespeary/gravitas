use std::collections::HashMap;

use crate::bytecode::{Chunk, Value};

pub enum RcOperation {
    Increment,
    Decrement,
}

#[derive(Default, Debug, Clone)]
struct EnvClosure {
    parent_env_key: usize,
    upvalues: Vec<Value>,
    reference_count: usize,
    referenced_environments: Vec<usize>,
}

impl EnvClosure {
    pub fn new(parent_env_key: usize, referenced_environments: Vec<usize>) -> Self {
        Self {
            parent_env_key,
            upvalues: vec![Value::Null],
            reference_count: 1,
            referenced_environments,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct Environments {
    // Reference to the parent node and vector of closed values
    closures: HashMap<usize, EnvClosure>,
}

impl Environments {
    pub fn new() -> Self {
        Self {
            // We start with a dummy closure for a global scope
            closures: {
                let mut map = HashMap::new();
                map.insert(0, EnvClosure::new(0, vec![]));
                map
            },
        }
    }

    pub fn create_env(
        &mut self,
        enclosing_env_key: usize,
        referenced_environments: Vec<usize>,
    ) -> usize {
        // We start from one, because 0 is reserved for global scope
        let key = self.closures.len() + 1;

        // Increment reference counts of the environments that this environment might point to
        for ref_env in &referenced_environments {
            self.decrement_rc(*ref_env);
        }

        self.closures.insert(
            key,
            EnvClosure::new(enclosing_env_key, referenced_environments),
        );
        key
    }

    pub fn close_value(&mut self, value: Value, env_key: usize) {
        let env = self
            .closures
            .get_mut(&env_key)
            .expect("Tried to access non-existent environment.");
        env.reference_count += 1;
        env.upvalues.push(value);
    }

    pub fn find_env(&self, mut env_key: usize, mut depth: usize) -> usize {
        while depth != 0 {
            env_key = self
                .closures
                .get(&env_key)
                .expect("We must have encountered global scope")
                .parent_env_key;

            depth -= 1;
        }
        env_key
    }

    pub fn get_value(
        &mut self,
        starting_env_key: usize,
        index: usize,
        depth: usize,
    ) -> Option<Value> {
        let env_key = self.find_env(starting_env_key, depth);
        self.closures
            .get(&env_key)
            .map(|closure| closure.upvalues.get(index).cloned())
            .flatten()
    }

    pub fn get_value_mut(
        &mut self,
        starting_env_key: usize,
        index: usize,
        depth: usize,
    ) -> Option<&mut Value> {
        let env_key = self.find_env(starting_env_key, depth);
        self.closures
            .get_mut(&env_key)
            .map(|closure| closure.upvalues.get_mut(index))
            .flatten()
    }

    pub fn change_rc(&mut self, env_key: usize, operation: RcOperation) {
        let closure = self
            .closures
            .get_mut(&env_key)
            .expect("Tried to increment reference count of non-existent environment");

        match operation {
            RcOperation::Decrement => {
                closure.reference_count -= 1;
            }
            RcOperation::Increment => {
                closure.reference_count += 1;
            }
        };
        dbg!(&self.closures);
    }

    pub fn increment_rc(&mut self, env_key: usize) {
        self.change_rc(env_key, RcOperation::Increment);
    }

    pub fn decrement_rc(&mut self, env_key: usize) {
        self.change_rc(env_key, RcOperation::Decrement);
    }

    pub fn destroy_env(&mut self, env_key: usize) {
        let env = self
            .closures
            .remove(&env_key)
            .expect("Tried to destroy non-existent environment");

        for ref_env in env.referenced_environments {
            self.decrement_rc(ref_env);
        }
    }
}

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub chunk: Chunk,
    pub stack_start: usize,
    pub return_address: usize,
    pub caller_name: String,
    pub env_key: usize,
}
