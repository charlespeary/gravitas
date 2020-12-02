use std::collections::HashMap;

use crate::bytecode::{Chunk, Value};
use crate::utils::graph::Graph;

#[derive(Default, Debug, Clone)]
pub struct Environments {
    closures: HashMap<usize, Vec<Value>>,
    connections: Graph<usize>,
}

impl Environments {
    pub fn create_env(&mut self, enclosing_env_key: usize) -> usize {
        // We start from one, because 0 is reserved for global scope
        let key = self.closures.len() + 1;
        self.closures.insert(key, vec![Value::Null]);
        self.connections.add((enclosing_env_key, key));
        key
    }

    pub fn close_value(&mut self, value: Value, env_key: usize) {
        self.closures
            .get_mut(&env_key)
            .expect("Tried to access non-existent environment.")
            .push(value);
    }

    pub fn find_env(&self, mut env_key: usize, mut depth: usize) -> usize {
        while depth != 0 {
            env_key = self.connections.get_left(&env_key).expect("test");
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
            .map(|env| env.get(index).cloned())
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
            .map(|env| env.get_mut(index))
            .flatten()
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
