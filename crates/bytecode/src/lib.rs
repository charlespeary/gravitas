use callables::Function;
use chunk::{Chunk, Constant, ConstantIndex};
use common::{ProgramText, MAIN_FUNCTION_NAME};
use parser::parse::Ast;
use state::GeneratorState;

pub mod callables;
pub mod chunk;
pub(crate) mod expr;
pub(crate) mod state;
pub(crate) mod stmt;

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct Patch {
    index: usize,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum MemoryAddress {
    // Local variables, e.g defined inside block or a function.
    // This value is added to the function's stack offset.
    Local(usize),
    // Upvalue address
    // First value points to the stack index that starts at index
    // defined by callstack n (second value) jumps above.
    Upvalue(usize, usize),
    // Global variable refereed by a string key.
    // The value is extracted from a HashMap of globals.
    // Note: all of the variables and functions defined in vtas are "local" per se.
    // Only the std functions are global.
    Global(String),
    // Property of an object
    // Property(PropertyAddress),
}

impl From<MemoryAddress> for Constant {
    fn from(address: MemoryAddress) -> Self {
        Constant::MemoryAddress(address)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: ProgramText,
    pub depth: usize,
    // Calculated index on the stack
    pub index: usize,
    // Flag to determine whether variable is used inside a closure and needs to be closed
    // in order to be available after it should go off the stack.
    pub closed: bool,
}

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
    // jump if false
    Jif(isize),
    // jump (both forwards or backwards)
    Jp(isize),
    // return
    Rtr,
    // pop n values from stack
    Pop(usize),
    // Get (Address)
    Get,
    // Assign (Address, Any)
    Asg,
    // Call function or method, (Callable)
    Call,
    // Return (Any)
    Return,
    Block(usize),
    Null,
}

impl Opcode {
    pub fn patch(self, value: isize) -> Self {
        match self {
            Opcode::Jif(_) => Opcode::Jif(value),
            Opcode::Jp(_) => Opcode::Jp(value),
            _ => unreachable!("Tried to patch invalid opcode"),
        }
    }
}

pub type BytecodeGenerationResult = Result<(), ()>;

#[derive(Debug, Clone)]
struct BytecodeGenerator {
    state: GeneratorState,
    functions: Vec<Function>,
}

impl BytecodeGenerator {
    pub fn new() -> Self {
        Self {
            state: GeneratorState::new(),
            functions: vec![Function {
                name: MAIN_FUNCTION_NAME.to_owned(),
                arity: 0,
                chunk: Chunk::default(),
            }],
        }
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.functions.last_mut().unwrap().chunk
    }

    pub fn write_opcode(&mut self, opcode: Opcode) -> usize {
        self.current_chunk().write_opcode(opcode)
    }

    pub fn write_constant(&mut self, constant: Constant) -> usize {
        self.current_chunk().write_constant(constant)
    }

    pub fn code(mut self) -> Function {
        if self.functions.len() > 1 {
            panic!("Tried to own the code before generation finished!");
        }

        let global_function = self
            .functions
            .pop()
            .expect("Generator is in invalid state!");

        global_function
    }

    pub fn curr_index(&mut self) -> usize {
        let size = self.current_chunk().opcodes.len();
        if size == 0 {
            0
        } else {
            size - 1
        }
    }

    pub fn emit_patch(&mut self, opcode: Opcode) -> Patch {
        let index = self.write_opcode(opcode);
        let patch = Patch { index };
        self.state.add_patch(patch);
        patch
    }

    pub fn patch(&mut self, patch: &Patch) {
        let current_index = self.curr_index();
        let opcode = self
            .current_chunk()
            .opcodes
            .get_mut(patch.index)
            .expect("Patch tried to access wrong opcode.");
        let patched_opcode =
            opcode.patch((current_index as f32 - patch.index as f32).abs() as isize);
        let _ = std::mem::replace(opcode, patched_opcode);
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

#[cfg(test)]
pub(crate) mod test {

    use crate::{chunk::Constant, BytecodeFrom, BytecodeGenerator, Opcode};

    pub(crate) fn assert_bytecode<D>(data: D, expected_bytecode: Vec<Opcode>)
    where
        BytecodeGenerator: BytecodeFrom<D>,
    {
        let mut generator = BytecodeGenerator::new();
        generator.generate(data).expect("Generation failed");
        assert_eq!(generator.code().chunk.opcodes, expected_bytecode)
    }

    pub(crate) fn assert_constants<D>(data: D, expected_constants: Vec<Constant>)
    where
        BytecodeGenerator: BytecodeFrom<D>,
    {
        let mut generator = BytecodeGenerator::new();
        generator.generate(data).expect("Generation failed");
        assert_eq!(generator.code().chunk.constants, expected_constants)
    }

    pub(crate) fn assert_bytecode_and_constants<D: Clone>(
        data: D,
        expected_bytecode: Vec<Opcode>,
        expected_constants: Vec<Constant>,
    ) where
        BytecodeGenerator: BytecodeFrom<D>,
    {
        assert_bytecode(data.clone(), expected_bytecode);
        assert_constants(data, expected_constants);
    }

    use parser::parse::{
        expr::{atom::AtomicValue, Expr, ExprKind},
        stmt::{Stmt, StmtKind},
        Node,
    };

    pub(crate) fn declare_var(name: String, expr: Expr) -> Stmt {
        Node {
            kind: Box::new(StmtKind::VariableDeclaration { name, expr }),
            span: 0..0,
        }
    }

    pub(crate) fn expr(atomic_value: AtomicValue) -> Expr {
        Node {
            kind: Box::new(ExprKind::Atom(atomic_value)),
            span: 0..0,
        }
    }

    pub(crate) fn expr_stmt(expr: Expr) -> Stmt {
        Node {
            kind: Box::new(StmtKind::Expression { expr }),
            span: 0..0,
        }
    }

    pub(crate) fn node<T>(kind: T) -> Node<T> {
        Node { kind, span: 0..0 }
    }

    pub(crate) fn box_node<T>(kind: T) -> Node<Box<T>> {
        node(Box::new(kind))
    }
}
