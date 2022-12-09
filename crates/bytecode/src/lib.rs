use callables::Function;
use chunk::{Chunk, ConstantIndex};
use parser::parse::Ast;
use state::GeneratorState;

pub mod callables;
pub mod chunk;
pub(crate) mod expr;
pub(crate) mod state;
pub(crate) mod stmt;

// Each opcode is described with e.g (Address, Number) which means that
// first Address followed by a Number will be popped from the stack.
// VM will panic if the popped value is not of an expected type.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Opcode {
    // Literals e.g number, string, bool
    Constant(ConstantIndex),
    // ! (Bool)
    Not,
    // - (Number)
    Neg,
    // + (Number, Number)
    Add,
    // - (Number, Number)
    Sub,
    // / (Number, Number)
    Div,
    // * (Number, Number)
    Mul,
    // ** (Number, Number)
    Pow,
    // % (Number, Number)
    Mod,
    // == (Any, Any)
    Eq,
    // != (Any, Any)
    Ne,
    // < (Number, Number)
    Lt,
    // <= (Number, Number)
    Le,
    // > (Number, Number)
    Gt,
    // >= (Number, Number)
    Ge,
    // or (Bool, Bool)
    Or,
    // and (Bool, Bool)
    And,
    // jump if false (Usize, Bool)
    Jif,
    // jump (Isize)
    Jp,
    // return
    Rtr,
    // pop n values from stack (Usize)
    Pop,
    // Get (Address)
    Get,
    // Assign (Address, Any)
    Asg,
    // Call function or method, (Callable)
    Call,
    // Return (Any)
    Return,
}

pub type BytecodeGenerationResult = Result<(), ()>;

struct BytecodeGenerator {
    chunk: Chunk,
    state: GeneratorState,
    functions: Vec<Function>,
}

impl BytecodeGenerator {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::default(),
            state: GeneratorState::default(),
            functions: vec! [
                Function {
                    name:
                }
            ],
        }
    }
}

pub trait BytecodeFrom<T> {
    fn generate(&mut self, data: T) -> BytecodeGenerationResult;
}

impl BytecodeFrom<Ast> for BytecodeGenerator {
    fn generate(&mut self, ast: Ast) -> BytecodeGenerationResult {
        for stmt in ast {
            self.generate(stmt)?;
        }
        Ok(())
    }
}

pub fn generate_bytecode(ast: Ast) -> BytecodeGenerationResult {
    let mut generator = BytecodeGenerator::new();
    generator.generate(ast)?;
    Ok(())
}
