use std::collections::HashMap;
use std::fmt;

use crate::{
    bytecode::{Chunk, Value},
    utils::logger::LOGGER,
};

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

impl fmt::Display for CallFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Env key: {}, Stack start: {}, Return address: {}, Caller: {}",
            self.env_key, self.stack_start, self.return_address, self.caller_name
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct Callstack {
    pub stack: Vec<CallFrame>,
    pub current_frame: Option<CallFrame>,
}

impl Callstack {
    pub fn new(initial_frame: CallFrame) -> Self {
        Self {
            stack: vec![initial_frame],
            current_frame: None,
        }
    }

    pub fn push(&mut self, frame: CallFrame) {
        LOGGER.log_dsp("CALLSTACK / PUSH", &frame);
        self.stack.push(frame);
    }

    pub fn next(&mut self) -> Option<CallFrame> {
        self.current_frame = self.stack.pop();
        LOGGER.log_title("CALLSTACK / POP");
        self.current_frame.clone()
    }

    pub fn current(&self) -> &CallFrame {
        self.current_frame
            .as_ref()
            .expect("Callstack tried to access frame from an empty stack!")
    }

    pub fn lookup(&self, offset: usize) -> &CallFrame {
        self.stack
            .get(self.stack.len() - offset)
            .expect("Callframe lookup failed!")
    }
}
