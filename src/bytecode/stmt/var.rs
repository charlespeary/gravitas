use anyhow::{Context, Result};

use crate::{
    bytecode::{Address, BytecodeFrom, BytecodeGenerator, GenerationResult, Opcode},
    parser::stmt::var::VarStmt,
    std::GLOBALS,
};

impl BytecodeGenerator {
    pub fn declare(&mut self, name: String) {
        self.locals.push(name);
        self.current_scope().declared += 1;
    }

    pub fn find(&self, name: &str) -> Result<Address> {
        self.locals
            .iter()
            .rposition(|l| l == name)
            .map(|local| Address::Local(local))
            .or_else(|| GLOBALS.get(name).map(|_| Address::Global(name.to_owned())))
            .with_context(|| format!("{} doesn't exist", name))
    }
}

impl BytecodeFrom<VarStmt> for BytecodeGenerator {
    fn generate(&mut self, var: &VarStmt) -> GenerationResult {
        let VarStmt { expr, identifier } = var;
        self.generate(expr)?;

        let identifier = identifier.clone();
        self.declare(identifier);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::{
        bytecode::{
            test::{into_bytecode, DECLARE_VAR, OMIT_VAR, VARIABLE_NAME},
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
            stmt::{print::PrintStmt, var::VarStmt, Stmt},
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
