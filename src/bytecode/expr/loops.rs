use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, GenerationResult, Opcode, Patch, PATCH},
    parser::expr::{Break, Continue, WhileLoop},
};

impl BytecodeFrom<WhileLoop> for BytecodeGenerator {
    fn generate(&mut self, wl: &WhileLoop) -> GenerationResult {
        let WhileLoop { condition, body } = wl;
        self.begin_loop();
        let start = self.curr_index();
        self.generate(condition.as_ref())?;

        let jif = self.emit_patch(Opcode::JumpIfFalse(PATCH));
        self.generate(body)?;

        self.emit_code(Opcode::PopN(1));
        let end = self.curr_index();
        self.emit_code(Opcode::JumpBack(end - start));
        self.patch(&jif);
        self.emit_code(Opcode::Null);

        let current_loop = self.end_loop();
        self.patch_many(&current_loop.patches);
        Ok(())
    }
}

impl BytecodeFrom<Continue> for BytecodeGenerator {
    fn generate(&mut self, _continue: &Continue) -> GenerationResult {
        let ending_index = self.curr_index();
        let starting_index = self.current_loop().starting_index;
        self.emit_code(Opcode::JumpBack(ending_index - starting_index));
        Ok(())
    }
}

impl BytecodeFrom<Break> for BytecodeGenerator {
    fn generate(&mut self, _break: &Break) -> GenerationResult {
        let Break { expr } = _break;
        if let Some(break_expr) = expr {
            self.generate(break_expr.as_ref())?;
        } else {
            self.emit_code(Opcode::Null);
        }
        let break_patch = self.emit_patch(Opcode::Break(PATCH));
        self.current_loop().patches.push(break_patch);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::{
        bytecode::{test::generate_bytecode, Value},
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
    fn while_expr() {
        let ast = WhileLoop {
            condition: Box::new(Expr::Binary(Binary {
                left: Box::new(Expr::Atom(Atom::Number(10.0))),
                operator: Operator::Less,
                right: Box::new(Expr::Atom(Atom::Number(20.0))),
            })),
            body: Block {
                body: vec![Stmt::Print(PrintStmt {
                    expr: Expr::Atom(Atom::Text(String::from("while loop"))),
                })],
                final_expr: None,
            },
        };

        let (_, bytecode) = generate_bytecode(ast);

        assert_eq!(
            bytecode,
            vec![
                Opcode::Constant(0),
                Opcode::Constant(1),
                Opcode::Less,
                Opcode::JumpIfFalse(5),
                Opcode::Constant(2),
                Opcode::Print,
                Opcode::Null,
                Opcode::PopN(1),
                Opcode::JumpBack(7),
                Opcode::Null,
            ]
        )
    }
}
