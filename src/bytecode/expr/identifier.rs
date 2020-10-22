use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, GenerationResult, Opcode, Patch, PATCH},
    parser::expr::Identifier,
};

impl BytecodeFrom<Identifier> for BytecodeGenerator {
    fn generate(&mut self, identifier: &Identifier) -> GenerationResult {
        let Identifier { is_ref, value } = identifier;
        let local = self.find_local(value)?;
        let opcode = match *is_ref {
            true => Opcode::VarRef(local),
            false => Opcode::Var(local),
        };
        self.emit_code(opcode);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use anyhow::{Context, Result};
    use pretty_assertions::assert_eq;

    use crate::bytecode::{
        test::{DECLARE_VAR, OMIT_VAR, VARIABLE_NAME},
        Chunk,
    };

    use super::*;

    fn generate_bytecode_with_var<I>(ast: I, should_declare: bool) -> Result<(Chunk, Vec<Opcode>)>
    where
        BytecodeGenerator: BytecodeFrom<I>,
    {
        let mut bg = BytecodeGenerator::new();
        if should_declare {
            bg.add_local(VARIABLE_NAME.to_owned());
        }
        bg.generate(&ast)
            .with_context(|| "Couldn't generate chunk from given ast")?;

        Ok((
            bg.chunk.clone(),
            bg.chunk.into_iter().cloned().collect::<Vec<Opcode>>(),
        ))
    }

    #[test]
    fn expr_var() -> Result<()> {
        // Bytecode generator will handle the Expr::Var if variable has been declared
        // and is stored in the locals vector.

        // Variables that evaluate to value
        let ast = Identifier {
            value: VARIABLE_NAME.to_owned(),
            is_ref: false,
        };

        let (_, bytecode) = generate_bytecode_with_var(ast, DECLARE_VAR)?;
        assert_eq!(bytecode, vec![Opcode::Var(0)]);

        // Variables that evaluate to reference
        let ast = Identifier {
            value: VARIABLE_NAME.to_owned(),
            is_ref: true,
        };

        let (_, bytecode) = generate_bytecode_with_var(ast, DECLARE_VAR)?;
        assert_eq!(bytecode, vec![Opcode::VarRef(0)]);

        // Bytecode generator will throw an error if variable referenced by Expr::Var hasn't been declared
        // and isn't stored in the locals vector.

        // Variables that evaluate to value
        let ast = Identifier {
            value: VARIABLE_NAME.to_owned(),
            is_ref: false,
        };
        assert!(generate_bytecode_with_var(ast, OMIT_VAR).is_err());

        // Variables that evaluate to reference
        let ast = Identifier {
            value: VARIABLE_NAME.to_owned(),
            is_ref: true,
        };
        assert!(generate_bytecode_with_var(ast, OMIT_VAR).is_err());

        Ok(())
    }
}
