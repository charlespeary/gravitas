use std::hash::Hash;

use anyhow::Result;

pub use chunk::Chunk;
pub use opcode::Opcode;
pub use value::{Address, Callable, Number, Value};

use crate::bytecode::state::{GeneratorState, ScopeType};
use crate::parser::Ast;

pub mod chunk;
pub mod expr;
pub mod opcode;
pub mod state;
pub mod stmt;
pub mod value;

pub type GenerationResult = Result<()>;

pub trait BytecodeFrom<T> {
    fn generate(&mut self, data: &T) -> GenerationResult;
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
    // Chunk with the whole program code
    chunk: Chunk,
    // Vec of chunks used to store functions code
    fn_chunks: Vec<Chunk>,
    // Vector of declared variables
    loops: Vec<Loop>,
    state: GeneratorState,
}

impl BytecodeGenerator {
    pub fn new() -> Self {
        Self {
            state: GeneratorState::new(),
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
        let size = self.current_chunk().size();
        if size == 0 {
            0
        } else {
            size - 1
        }
    }

    pub fn current_chunk(&mut self) -> &mut Chunk {
        self.fn_chunks.last_mut().unwrap_or(&mut self.chunk)
    }

    pub fn emit_codes(&mut self, opcodes: Vec<Opcode>) -> usize {
        let length = opcodes.len();

        for opcode in opcodes {
            self.emit_code(opcode);
        }

        length
    }

    pub fn emit_code(&mut self, opcode: Opcode) -> usize {
        self.current_chunk().grow(opcode)
    }

    // OPCODE PATCHING
    pub fn emit_patch(&mut self, opcode: Opcode) -> Patch {
        let index = self.emit_code(opcode);
        Patch { index }
    }

    pub fn patch(&mut self, patch: &Patch) {
        let current_index = self.curr_index();
        let opcode = self
            .current_chunk()
            .code
            .get_mut(patch.index)
            .expect("Patch tried to access wrong opcode.");
        let patched_opcode =
            opcode.patch((current_index as f32 - patch.index as f32).abs() as usize);
        let _ = std::mem::replace(opcode, patched_opcode);
    }

    pub fn patch_many(&mut self, patches: &[Patch]) {
        for patch in patches {
            self.patch(patch);
        }
    }

    pub fn close_scope_variables(&mut self) {
        let closed_values: Vec<(Address, Opcode)> = self
            .state
            .scope_closed_variables()
            .iter()
            .map(|var| (Address::Local(var.index), Opcode::CloseValue))
            .collect();

        for (address, opcode) in closed_values {
            self.add_constant(address.into());
            self.emit_code(opcode);
        }
    }

    pub fn pop_scope_variables(&mut self) {
        let variables_len = self.state.declared();
        self.emit_code(Opcode::PopN(variables_len));
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

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.current_chunk().add_constant(value)
    }

    pub fn enter_callable(&mut self, scope_type: ScopeType) {
        self.state.enter_scope(scope_type);
        self.fn_chunks.push(Chunk::default());
    }

    pub fn emit_return(&mut self) {
        if !self.state.did_return() {
            self.close_scope_variables();
            self.emit_code(Opcode::Null);
            self.emit_code(Opcode::Return);
        }
        self.state.leave_scope();
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
