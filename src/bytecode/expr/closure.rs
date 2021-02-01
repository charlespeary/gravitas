use crate::{
    bytecode::{
        state::ScopeType, BytecodeFrom, BytecodeGenerator, Callable, Chunk, GenerationResult,
        Opcode, Value,
    },
    parser::{expr::Closure as ClosureExpr, Expr},
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ClosureAddress {
    // How many call frames above the value is
    pub depth: usize,
    // Index of the value on the stack
    pub index: usize,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Closure {
    pub chunk: Chunk,
    pub arity: usize,
    pub env_key: Option<usize>,
    pub enclosing_env_key: Option<usize>,
    pub referenced_environments: Vec<usize>,
}

impl Into<Value> for Closure {
    fn into(self) -> Value {
        Value::Callable(Callable::Closure(self))
    }
}

impl Closure {
    pub fn with_env(mut self, env_key: usize) -> Self {
        self.env_key = Some(env_key);
        self
    }

    pub fn with_enclosing_env(mut self, enclosing_env_key: usize) -> Self {
        self.enclosing_env_key = Some(enclosing_env_key);
        self
    }
}

impl BytecodeFrom<ClosureExpr> for BytecodeGenerator {
    fn generate(&mut self, closure: &ClosureExpr) -> GenerationResult {
        let ClosureExpr { body, params } = closure;
        self.enter_callable(ScopeType::Closure);

        for param in params.clone() {
            self.state.declare_var(&param.val);
        }

        // Declare a name for anonymous function, so the referenced variables start from the index 1, not 0
        self.state.declare_var("lambda");

        match *body.clone() {
            Expr::Block(block) => {
                for item in &block.body {
                    self.generate(item)?;
                }
            }
            body => {
                self.generate(&body)?;
            }
        }

        // Vector of environments above that contain values referenced by this closure
        let referenced_environments: Vec<usize> = self
            .state
            .scope_closed_variables()
            .iter()
            .map(|v| v.depth)
            .collect();

        self.emit_return();

        let lambda = Closure {
            arity: params.len(),
            chunk: self
                .fn_chunks
                .pop()
                .expect("Tried to pop function's chunk that doesn't exist."),
            referenced_environments,
            env_key: None,
            enclosing_env_key: None,
        };

        self.add_constant(Value::Callable(Callable::Closure(lambda)));
        self.emit_code(Opcode::CreateClosure);
        Ok(())
    }
}
