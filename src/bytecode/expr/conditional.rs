use anyhow::Result;

use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, GenerationResult, Opcode, Patch, PATCH},
    parser::expr::conditional::{If, IfBranch},
};

impl BytecodeGenerator {
    fn evaluate_branch(&mut self, branch: &IfBranch) -> Result<Patch> {
        let IfBranch {
            condition, body, ..
        } = branch;
        self.generate(condition)?;
        let patch = self.emit_patch(Opcode::JumpIfFalse(PATCH));
        self.generate(body)?;
        let jump_forward = self.emit_patch(Opcode::JumpForward(PATCH));
        self.patch(&patch);
        Ok(jump_forward)
    }
}

impl BytecodeFrom<If> for BytecodeGenerator {
    fn generate(&mut self, conditional: &If) -> GenerationResult {
        let If { branches } = conditional;
        let branches_patches: Vec<Patch> = branches
            .iter()
            .map(|b| self.evaluate_branch(b))
            .collect::<Result<Vec<Patch>>>()?;
        self.emit_code(Opcode::Null);
        self.patch_many(&branches_patches);
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
                block::Block,
                conditional::{BranchType, If, IfBranch},
                Expr,
            },
            stmt::{var::VarStmt, Stmt},
        },
    };

    use super::*;

    #[test]
    fn if_expr_if() {
        let ast = If {
            branches: vec![IfBranch {
                branch_type: BranchType::If,
                condition: Expr::Atom(Atom::Bool(true)),
                body: Block {
                    body: vec![Stmt::Var(VarStmt {
                        identifier: String::from("foo"),
                        expr: Expr::Atom(Atom::Bool(true)),
                    })],
                    final_expr: None,
                },
            }],
        };
        let (_, bytecode) = generate_bytecode(ast);
        assert_eq!(
            bytecode,
            vec![
                Opcode::True,
                Opcode::JumpIfFalse(4),
                Opcode::True,
                Opcode::Null,
                Opcode::Block(1),
                Opcode::JumpForward(1),
                Opcode::Null
            ]
        )
    }

    #[test]
    fn if_expr_elif() {
        let ast = If {
            branches: vec![
                IfBranch {
                    branch_type: BranchType::If,
                    condition: Expr::Atom(Atom::Bool(false)),
                    body: Block {
                        body: vec![Stmt::Var(VarStmt {
                            identifier: String::from("foo"),
                            expr: Expr::Atom(Atom::Bool(true)),
                        })],
                        final_expr: None,
                    },
                },
                IfBranch {
                    branch_type: BranchType::ElseIf,
                    condition: Expr::Atom(Atom::Bool(true)),
                    body: Block {
                        body: vec![Stmt::Var(VarStmt {
                            identifier: String::from("bar"),
                            expr: Expr::Atom(Atom::Bool(true)),
                        })],
                        final_expr: None,
                    },
                },
            ],
        };
        let (_, bytecode) = generate_bytecode(ast);
        assert_eq!(
            bytecode,
            vec![
                Opcode::False,
                Opcode::JumpIfFalse(4),
                Opcode::True,
                Opcode::Null,
                Opcode::Block(1),
                Opcode::JumpForward(7),
                Opcode::True,
                Opcode::JumpIfFalse(4),
                Opcode::True,
                Opcode::Null,
                Opcode::Block(1),
                Opcode::JumpForward(1),
                Opcode::Null
            ]
        )
    }

    #[test]
    fn if_expr_else() {
        let ast = If {
            branches: vec![
                IfBranch {
                    branch_type: BranchType::If,
                    condition: Expr::Atom(Atom::Bool(false)),
                    body: Block {
                        body: vec![Stmt::Var(VarStmt {
                            identifier: String::from("foo"),
                            expr: Expr::Atom(Atom::Bool(true)),
                        })],
                        final_expr: None,
                    },
                },
                IfBranch {
                    branch_type: BranchType::Else,
                    // Parser always makes else have a truthful condition
                    condition: Expr::Atom(Atom::Bool(true)),
                    body: Block {
                        body: vec![Stmt::Var(VarStmt {
                            identifier: String::from("bar"),
                            expr: Expr::Atom(Atom::Bool(true)),
                        })],
                        final_expr: None,
                    },
                },
            ],
        };
        let (_, bytecode) = generate_bytecode(ast);
        assert_eq!(
            bytecode,
            vec![
                Opcode::False,
                Opcode::JumpIfFalse(4),
                Opcode::True,
                Opcode::Null,
                Opcode::Block(1),
                Opcode::JumpForward(7),
                Opcode::True,
                Opcode::JumpIfFalse(4),
                Opcode::True,
                Opcode::Null,
                Opcode::Block(1),
                Opcode::JumpForward(1),
                Opcode::Null
            ]
        )
    }
}
