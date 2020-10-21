use anyhow::Result;

pub use chunk::Chunk;
pub use opcode::Opcode;
pub use value::{Address, Number, Value};

use crate::parser::Ast;

pub mod chunk;
pub mod expr;
pub mod opcode;
pub mod stmt;
pub mod value;

pub type GenerationResult = Result<()>;

pub trait BytecodeFrom<T> {
    fn generate(&mut self, data: &T) -> GenerationResult;
}

/// State of the scope / block
#[derive(Default, Debug, Copy, Clone)]
pub struct Scope {
    /// Amount of declared variables in the given scope.
    pub declared: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Loop {
    starting_index: usize,
    // Number of continue and break expressions in given loop
    patches: Vec<Patch>,
}

const PATCH: usize = 0;

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
pub struct Patch {
    index: usize,
}

#[derive(Default)]
pub struct BytecodeGenerator {
    chunk: Chunk,
    locals: Vec<String>,
    scopes: Vec<Scope>,
    loops: Vec<Loop>,
}

impl BytecodeGenerator {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::default()],
            ..Default::default()
        }
    }

    pub fn compile<I>(ast: &I) -> Result<Chunk>
    where
        Self: BytecodeFrom<I>,
    {
        let mut emitter = BytecodeGenerator::new();
        emitter.generate(ast)?;

        // temporary clone until I figure out how to generate bytecode properly
        Ok(emitter.chunk)
    }

    pub fn curr_index(&mut self) -> usize {
        let size = self.chunk.size();
        if size == 0 {
            0
        } else {
            size - 1
        }
    }

    pub fn emit_code(&mut self, opcode: Opcode) -> usize {
        self.chunk.grow(opcode)
    }

    // OPCODE PATCHING
    pub fn emit_patch(&mut self, opcode: Opcode) -> Patch {
        let index = self.emit_code(opcode);
        Patch { index }
    }

    pub fn patch(&mut self, patch: &Patch) {
        let current_index = self.curr_index();

        let opcode = self
            .chunk
            .code
            .get_mut(patch.index)
            .expect("Patch tried to access wrong opcode.");
        println!(
            "Current index: {} Patch index: {}",
            current_index, patch.index
        );
        let patched_opcode = opcode.patch(current_index - patch.index);
        let _ = std::mem::replace(opcode, patched_opcode);
    }

    pub fn patch_many(&mut self, patches: &[Patch]) {
        for patch in patches {
            self.patch(patch);
        }
    }

    pub fn begin_scope(&mut self) {
        self.scopes.push(Scope::default())
    }

    pub fn end_scope(&mut self) {
        let scope = self
            .scopes
            .pop()
            .expect("Tried to pop scope that doesn't exist");
        // Pop locals from given scope
        for _ in 0..scope.declared {
            self.locals.pop();
        }
        if scope.declared > 0 {
            self.emit_code(Opcode::Block(scope.declared));
        }
    }

    pub fn begin_loop(&mut self) -> usize {
        let starting_index = self.curr_index();
        self.loops.push(Loop {
            starting_index,
            patches: vec![],
        });
        self.loops.len()
    }

    pub fn end_loop(&mut self) -> Loop {
        self.loops
            .pop()
            .expect("Bytecode emitter is in invalid state. Tried to pop loop in no-loop context.")
    }

    pub fn current_loop(&mut self) -> &mut Loop {
        // Static analysis will ensure that we won't ever generate bytecode
        // that will contain code meant for loops placed outside the loops, so
        // we can safely unwrap this.
        self.loops.last_mut().unwrap()
    }
}

impl From<&BytecodeGenerator> for BytecodeGenerator {
    fn from(outer: &BytecodeGenerator) -> Self {
        BytecodeGenerator {
            locals: outer.locals.clone(),
            scopes: outer.scopes.clone(),
            loops: outer.loops.clone(),
            ..Default::default()
        }
    }
}

impl BytecodeFrom<Ast> for BytecodeGenerator {
    fn generate(&mut self, ast: &Ast) -> GenerationResult {
        for stmt in &ast.0 {
            self.generate(stmt)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    pub const VARIABLE_NAME: &str = "foo";
    pub const DECLARE_VAR: bool = true;
    pub const OMIT_VAR: bool = false;

    pub fn into_bytecode(chunk: Chunk) -> Vec<Opcode> {
        chunk.into_iter().cloned().collect::<Vec<Opcode>>()
    }

    pub fn generate_bytecode<I>(ast: I) -> (Chunk, Vec<Opcode>)
    where
        BytecodeGenerator: BytecodeFrom<I>,
    {
        let chunk =
            BytecodeGenerator::compile(&ast).expect("Couldn't generate chunk from given ast");
        (chunk.clone(), into_bytecode(chunk))
    }
}
