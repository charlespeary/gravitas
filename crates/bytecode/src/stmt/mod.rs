use std::fmt::Display;

use crate::{
    callables::{Class, Function},
    chunk::Constant,
    state::ScopeType,
    BytecodeFrom, BytecodeGenerationResult, BytecodeGenerator, MemoryAddress, Opcode,
};
use common::CONSTRUCTOR_NAME;
use parser::parse::{
    expr::ExprKind,
    stmt::{Stmt, StmtKind},
    FunctionBody, Params,
};

mod var;

pub type GlobalPointer = usize;

#[derive(Debug, Clone)]
pub enum GlobalItem {
    Function(Function),
    Class(Class),
}

impl GlobalItem {
    pub fn name(&self) -> &String {
        match self {
            GlobalItem::Function(function) => &function.name,
            GlobalItem::Class(class) => &class.name,
        }
    }
}

impl GlobalItem {
    pub fn as_function(&self) -> &Function {
        match self {
            GlobalItem::Function(function) => function,
            _ => panic!("Expected function, got class"),
        }
    }

    pub fn as_class(&self) -> &Class {
        match self {
            GlobalItem::Class(class) => class,
            _ => panic!("Expected class, got function"),
        }
    }
}

impl From<Function> for GlobalItem {
    fn from(function: Function) -> Self {
        GlobalItem::Function(function)
    }
}

impl From<Class> for GlobalItem {
    fn from(class: Class) -> Self {
        GlobalItem::Class(class)
    }
}

impl Display for GlobalItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlobalItem::Function(function) => write!(f, "{}", function),
            GlobalItem::Class(class) => write!(f, "{}", class),
        }
    }
}

impl BytecodeGenerator {
    pub(crate) fn compile_function(
        &mut self,
        name: String,
        params: Params,
        body: FunctionBody,
    ) -> Result<Function, ()> {
        self.new_function(name.clone(), params.kind.len());

        for param in params.kind {
            self.state.declare_var(param.kind);
        }

        // To allow recursion
        self.state.declare_var(name.clone());

        match *body.kind {
            ExprKind::Block { stmts, return_expr } => {
                self.generate(stmts)?;
                match return_expr {
                    Some(return_expr) => {
                        self.generate(return_expr)?;
                    }
                    None => {
                        self.write_opcode(Opcode::Null);
                    }
                };
                self.write_opcode(Opcode::Return);
            }
            _ => {
                self.generate(body)?;
                self.write_opcode(Opcode::Return);
            }
        };

        let new_fn = self
            .functions
            .pop()
            .expect("We just defined and evaluated function. It shouldn't happen.");
        self.leave_scope();

        return Ok(new_fn);
    }

    pub fn declare_global(&mut self, item: GlobalItem) -> GlobalPointer {
        self.state.declare_var(item.name().clone());
        self.globals.push(item);
        self.globals.len() - 1
    }
}

impl BytecodeFrom<Stmt> for BytecodeGenerator {
    fn generate(&mut self, stmt: Stmt) -> BytecodeGenerationResult {
        match *stmt.kind {
            StmtKind::Expression { expr } => {
                self.generate(expr)?;
            }
            StmtKind::VariableDeclaration { name, expr } => {
                self.generate(expr)?;
                self.state.declare_var(name);
            }
            StmtKind::FunctionDeclaration { name, params, body } => {
                let new_fn = self.compile_function(name.clone(), params, body)?;
                let fn_ptr = self.declare_global(new_fn.into());

                let (upvalues_addresses, upvalues_count) = {
                    let upvalues = self.state.scope_upvalues();
                    println!("Function {} has {} upvalues", name, upvalues.len());
                    let count = upvalues.len();
                    let addresses: Vec<Constant> = upvalues
                        .iter()
                        .map(|upvalue| {
                            println!("Upvalue: {:?}", upvalue);
                            // It's still on the stack because depth 1 means that it's the function in which closure is declared
                            if upvalue.is_local {
                                Constant::MemoryAddress(MemoryAddress::Local(upvalue.local_index))
                            } else {
                                Constant::MemoryAddress(MemoryAddress::Upvalue(
                                    upvalue.upvalue_index,
                                ))
                            }
                        })
                        .collect();

                    (addresses, count)
                };

                self.write_constant(Constant::GlobalPointer(fn_ptr));

                for upvalue_address in upvalues_addresses {
                    self.write_constant(upvalue_address);
                }

                self.write_opcode(Opcode::CreateClosure(upvalues_count));
            }
            StmtKind::ClassDeclaration {
                name,
                super_class,
                methods,
            } => {
                self.enter_scope(ScopeType::Class);

                // To allow methods calling class constructor
                self.state.declare_var(name.clone());

                let mut compiled_methods = vec![];
                let mut constructor: Option<GlobalPointer> = None;

                for method in methods {
                    if let StmtKind::FunctionDeclaration { name, params, body } = *method.kind {
                        let is_constructor = name == CONSTRUCTOR_NAME;
                        let compiled_method = self.compile_function(name, params, body)?;
                        let method_ptr = self.declare_global(compiled_method.into());

                        if is_constructor {
                            constructor = Some(method_ptr);
                        } else {
                            compiled_methods.push(method_ptr);
                        }
                    } else {
                        panic!("Analyzer didn't prevent items that are not function declarations from getting there!");
                    }
                }

                self.leave_scope();

                let class = Class {
                    name,
                    super_class: None,
                    methods: compiled_methods,
                    constructor,
                };

                let class_ptr = self.declare_global(class.into());
                self.write_constant(Constant::GlobalPointer(class_ptr));
            }
        }
        Ok(())
    }
}
