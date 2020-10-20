use anyhow::{Context, Result};

use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, GenerationResult},
    parser::stmt::var::VarStmt,
};

impl BytecodeGenerator {
    pub fn add_local(&mut self, name: String) {
        self.locals.push(name);
        self.scopes.last_mut().map_or_else(
            || panic!("Couldn't access current scope!"),
            |s| {
                s.declared += 1;
            },
        );
    }

    pub fn find_local(&self, name: &str) -> Result<usize> {
        self.locals
            .iter()
            .rposition(|l| l == name)
            .with_context(|| format!("{} doesn't exist", name))
    }
}

impl BytecodeFrom<VarStmt> for BytecodeGenerator {
    fn generate(&mut self, var: &VarStmt) -> GenerationResult {
        let VarStmt { expr, identifier } = var;
        self.generate(expr)?;
        self.add_local(identifier.clone());
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

        let chunk = bg
            .compile(&ast)
            .expect("Couldn't generate bytecode for given ast");

        let bytecode = into_bytecode(chunk.clone());

        // Bytecode generator adds newly created variable to the locals vector,
        // so it can remember and figure out where variables should be stored on stack.
        // We can search for given local and get back its index on the stack wrapped in a Result.
        // Error is thrown if variable was not created and therefore doesn't exist.
        bg.begin_scope();
        bg.add_local(VARIABLE_NAME.to_owned());
        assert_eq!(bg.locals, vec![VARIABLE_NAME.to_owned()]);
        assert_eq!(
            bg.find_local(VARIABLE_NAME)
                .expect("Variable not found in the vector of local variables."),
            0
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
