use anyhow::{Context, Result};

use crate::{
    bytecode::{Address, BytecodeFrom, BytecodeGenerator, GenerationResult, Opcode},
    parser::stmt::var::VarStmt,
    std::GLOBALS,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: String,
    pub depth: usize,
    // Calculated index on the stack
    pub index: usize,
    // Flag to determine whether variable is used inside a closure and needs to be closed
    // in order to be available after it should go off the stack.
    pub closed: bool,
}

#[derive(Default, Debug, Clone, PartialOrd, PartialEq)]
pub struct Upvalue {
    pub scopes_above: usize,
    pub index: usize,
}

impl BytecodeGenerator {
    // pub fn declare(&mut self, name: String) {
    //     self.state.declare_var(Variable {
    //         name,
    //         closed: false,
    //     })
    // }
    //
    // pub fn create_upvalue(&mut self, index: usize, depth: usize) -> Address {
    //     let upvalue_index = self.current_scope().upvalues.len();
    //     self.current_scope_mut()
    //         .upvalues
    //         .push(Upvalue { index, depth });
    //     Address::Upvalue(upvalue_index)
    // }

    pub fn find_var(&mut self, name: &str) -> Result<Address> {
        let current_depth = self.state.depth();
        self.state
            .find_var(name)
            .map(|var| {
                if var.closed {
                    Address::Upvalue(var.index, current_depth - var.depth)
                } else {
                    Address::Local(var.index)
                }
            })
            .or_else(|| GLOBALS.get(name).map(|_| Address::Global(name.to_owned())))
            .with_context(|| format!("{} doesn't exist", name))
        // We start by looking for a variable in local variables
        // self.declared
        //     .iter()
        //     .rposition(|v| v.name == name)
        //     .map(|i| {
        //         let var = self
        //             .declared
        //             .get_mut(i)
        //             .expect("We just found this variable");
        //         let depth = self.state.depth();
        //         if self.state.is_in_closure() && depth != var.depth {
        //             var.closed = true;
        //             self.create_upvalue(i, var.depth)
        //         } else {
        //             Address::Local(i)
        //         }
        //     })
        //     // in the end we fallback to globals values
        //     .or_else(|| GLOBALS.get(name).map(|_| Address::Global(name.to_owned())))
        //     .with_context(|| format!("{} doesn't exist", name))
    }
}

impl BytecodeFrom<VarStmt> for BytecodeGenerator {
    fn generate(&mut self, var: &VarStmt) -> GenerationResult {
        let VarStmt { expr, identifier } = var;
        self.generate(expr)?;
        self.state.declare_var(identifier);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::{
        bytecode::{
            test::{into_bytecode, OMIT_VAR, VARIABLE_NAME},
            Opcode, Value,
        },
        parser::{
            expr::{
                atom::Atom,
                binary::Binary,
                block::Block,
                conditional::{BranchType, If, IfBranch},
                Expr, Operator,
            },
            stmt::{var::VarStmt, Stmt},
        },
    };

    use super::*;

    #[test]
    fn stmt_var() {
        let mut bg = BytecodeGenerator::default();

        let ast = Block {
            body: vec![Stmt::Var(VarStmt {
                identifier: String::from(VARIABLE_NAME),
                expr: Expr::Atom(Atom::Number(10.0)),
            })],
            final_expr: None,
        };

        let chunk =
            BytecodeGenerator::compile(&ast).expect("Couldn't generate bytecode for given ast");

        let bytecode = into_bytecode(chunk.clone());

        // Bytecode generator adds newly created variable to the locals vector,
        // so it can remember and figure out where variables should be stored on stack.
        // We can search for given local and get back its index on the stack wrapped in a Result.
        // Error is thrown if variable was not created and therefore doesn't exist.
        bg.begin_scope();
        bg.declare(VARIABLE_NAME.to_owned());
        assert_eq!(bg.locals, vec![VARIABLE_NAME.to_owned()]);
        assert_eq!(
            bg.find(VARIABLE_NAME)
                .expect("Variable not found in the vector of local variables."),
            Address::Local(0)
        );
        bg.end_scope();
        // Variable declaration doesn't add any opcode overhead, because all variables are just temporary values on the stack.
        assert_eq!(
            bytecode,
            vec![Opcode::Constant(0), Opcode::Null, Opcode::Block(1)]
        );
        assert_eq!(chunk.read_constant(0), &Value::Number(10.0));
    }
}
