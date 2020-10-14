use anyhow::{Context, Result};

use crate::parser::{Ast, Atom, Block, Expr, IfBranch, Stmt, Token, Visitable, Visitor};
pub use chunk::Chunk;
pub use function::Function;
pub use opcode::Opcode;
pub use value::{Address, Number, Value};

mod chunk;
mod function;
mod opcode;
mod value;

/// State of the scope / block
#[derive(Default, Debug, Copy, Clone)]
pub struct Scope {
    /// Amount of declared variables in the given scope.
    pub declared: usize,
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
    chunk: Chunk,
    locals: Vec<String>,
    scopes: Vec<Scope>,
    loops: Vec<Loop>,
}

impl BytecodeGenerator {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::default()],
            ..Default::default()
        }
    }

    pub fn generate<I>(&mut self, ast: &I) -> Result<Chunk>
    where
        I: Visitable,
        Self: Visitor<I>,
    {
        ast.accept(self)?;

        // temporary clone until I figure out how to generate bytecode properly
        Ok(self.chunk.clone())
    }

    pub fn curr_index(&mut self) -> usize {
        let size = self.chunk.size();
        if size == 0 {
            0
        } else {
            size - 1
        }
    }

    pub fn emit_code(&mut self, opcode: Opcode) -> usize {
        self.chunk.grow(opcode)
    }

    // OPCODE PATCHING
    pub fn emit_patch(&mut self, opcode: Opcode) -> Patch {
        let index = self.emit_code(opcode);
        Patch { index }
    }

    pub fn patch(&mut self, patch: &Patch) {
        let current_index = self.curr_index();

        let opcode = self
            .chunk
            .code
            .get_mut(patch.index)
            .expect("Patch tried to access wrong opcode.");
        println!(
            "Current index: {} Patch index: {}",
            current_index, patch.index
        );
        let patched_opcode = opcode.patch(current_index - patch.index);
        let _ = std::mem::replace(opcode, patched_opcode);
    }

    pub fn patch_many(&mut self, patches: &[Patch]) {
        for patch in patches {
            self.patch(patch);
        }
    }

    pub fn begin_scope(&mut self) {
        self.scopes.push(Scope::default())
    }

    pub fn end_scope(&mut self) {
        let scope = self
            .scopes
            .pop()
            .expect("Tried to pop scope that doesn't exist");
        // Pop locals from given scope
        for _ in 0..scope.declared {
            self.locals.pop();
        }
        if scope.declared > 0 {
            self.emit_code(Opcode::Block(scope.declared));
        }
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

    fn evaluate_branch(&mut self, branch: &IfBranch) -> Result<Patch> {
        branch.condition.accept(self)?;
        let patch = self.emit_patch(Opcode::JumpIfFalse(PATCH));
        branch.body.accept(self)?;
        let jump_forward = self.emit_patch(Opcode::JumpForward(PATCH));
        println!("BRANCH");
        self.patch(&patch);
        Ok(jump_forward)
    }
}

impl From<&BytecodeGenerator> for BytecodeGenerator {
    fn from(outer: &BytecodeGenerator) -> Self {
        BytecodeGenerator {
            locals: outer.locals.clone(),
            scopes: outer.scopes.clone(),
            loops: outer.loops.clone(),
            ..Default::default()
        }
    }
}

impl Visitor<Ast> for BytecodeGenerator {
    type Item = ();
    fn visit(&mut self, ast: &Ast) -> Result<Self::Item> {
        for stmt in &ast.0 {
            stmt.accept(self)?;
        }
        Ok(())
    }
}

impl Visitor<Expr> for BytecodeGenerator {
    type Item = ();

    fn visit(&mut self, expr: &Expr) -> Result<Self::Item> {
        match expr {
            Expr::Atom(atom) => match atom {
                Atom::Number(num) => {
                    self.chunk.add_constant(Value::Number(*num));
                }
                Atom::Bool(bool) => {
                    self.emit_code((*bool).into());
                }
                Atom::Null => {
                    self.emit_code(Opcode::Null);
                }
                Atom::Text(string) => {
                    self.chunk.add_constant(Value::String(string.clone()));
                }
            },
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                left.accept(self)?;
                right.accept(self)?;
                self.emit_code(operator.clone().into());
            }
            Expr::Grouping { expr } => {
                expr.accept(self)?;
            }
            Expr::Unary { expr, operator } => {
                expr.accept(self)?;
                let opcode = match operator {
                    Token::Bang => Opcode::Not,
                    Token::Minus => Opcode::Negate,
                    _ => unreachable!(),
                };
                self.emit_code(opcode);
            }
            Expr::Var { identifier, is_ref } => {
                let local = self.find_local(identifier)?;
                let opcode = match *is_ref {
                    true => Opcode::VarRef(local),
                    false => Opcode::Var(local),
                };

                self.emit_code(opcode);
            }
            Expr::Block { body } => {
                body.accept(self)?;
            }
            Expr::If { branches } => {
                let branches_patches: Vec<Patch> = branches
                    .iter()
                    .map(|b| self.evaluate_branch(b))
                    .collect::<Result<Vec<Patch>>>()?;
                self.emit_code(Opcode::Null);
                println!("JUMP FORWARD");
                self.patch_many(&branches_patches);
            }
            Expr::Break { expr } => {
                if let Some(break_expr) = expr {
                    break_expr.accept(self)?;
                } else {
                    self.emit_code(Opcode::Null);
                }
                let break_patch = self.emit_patch(Opcode::Break(PATCH));
                self.current_loop().patches.push(break_patch);
            }

            Expr::Continue => {
                let ending_index = self.curr_index();
                let starting_index = self.current_loop().starting_index;
                self.emit_code(Opcode::JumpBack(ending_index - starting_index));
            }
            Expr::While { body, condition } => {
                self.begin_loop();
                let start = self.curr_index();
                condition.accept(self)?;

                let jif = self.emit_patch(Opcode::JumpIfFalse(PATCH));
                body.accept(self)?;

                self.emit_code(Opcode::PopN(1));
                let end = self.curr_index();
                self.emit_code(Opcode::JumpBack(end - start));
                self.patch(&jif);
                self.emit_code(Opcode::Null);

                let current_loop = self.end_loop();
                self.patch_many(&current_loop.patches);
            }
        }
        Ok(())
    }
}

impl Visitor<Stmt> for BytecodeGenerator {
    type Item = ();

    fn visit(&mut self, stmt: &Stmt) -> Result<Self::Item> {
        match stmt {
            // TODO: Delete this statement when concept of std lib is done
            Stmt::Print { expr } => {
                expr.accept(self)?;
                self.emit_code(Opcode::Print);
            }
            Stmt::Expr { expr } => {
                expr.accept(self)?;
                self.emit_code(Opcode::PopN(1));
            }
            Stmt::Var { expr, identifier } => {
                expr.accept(self)?;
                self.add_local(identifier.clone());
            }
            Stmt::Function { name, args, body } => {}
        }
        Ok(())
    }
}

impl Visitor<Block> for BytecodeGenerator {
    type Item = ();

    fn visit(&mut self, block: &Block) -> Result<Self::Item> {
        let Block { body, final_expr } = block;
        self.begin_scope();

        for item in body {
            item.accept(self)?;
        }

        match final_expr {
            Some(expr) => {
                expr.accept(self)?;
            }
            _ => {
                self.emit_code(Opcode::Null);
            }
        }
        self.end_scope();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{Block, BranchType};

    use super::*;

    fn into_bytecode(chunk: Chunk) -> Vec<Opcode> {
        chunk.into_iter().cloned().collect::<Vec<Opcode>>()
    }

    fn generate_bytecode<I>(ast: I) -> (Chunk, Vec<Opcode>)
    where
        I: Visitable,
        BytecodeGenerator: Visitor<I>,
    {
        let mut bg = BytecodeGenerator::new();
        let chunk = bg
            .generate(&ast)
            .expect("Couldn't generate chunk from given ast");
        (chunk.clone(), into_bytecode(chunk))
    }

    #[quickcheck]
    fn expr_atom_numbers(a: f64) {
        let ast = Expr::Atom(Atom::Number(a));
        let (chunk, bytecode) = generate_bytecode(ast);

        assert_eq!(bytecode, vec![Opcode::Constant(0)]);
        assert_eq!(*chunk.read_constant(0), Value::Number(a));
    }

    #[test]
    fn expr_atom_boolean() {
        let ast = Expr::Atom(Atom::Bool(true));
        let (_, bytecode) = generate_bytecode(ast);
        assert_eq!(bytecode, vec![Opcode::True]);

        let ast = Expr::Atom(Atom::Bool(false));
        let (_, bytecode) = generate_bytecode(ast);
        assert_eq!(bytecode, vec![Opcode::False]);
    }

    #[test]
    fn expr_atom_null() {
        let ast = Expr::Atom(Atom::Null);
        let (_, bytecode) = generate_bytecode(ast);
        assert_eq!(bytecode, vec![Opcode::Null]);
    }

    #[quickcheck]
    fn expr_atom_text(text: String) {
        let ast = Expr::Atom(Atom::Text(text.clone()));
        let (chunk, bytecode) = generate_bytecode(ast);
        assert_eq!(bytecode, vec![Opcode::Constant(0)]);
        assert_eq!(*chunk.read_constant(0), Value::String(text));
    }

    #[quickcheck]
    fn expr_binary(a: f64, b: f64) {
        macro_rules! test_binary_with_operators (
            ($a: expr, $b: expr, $($operator: expr),*) => {
                $(
                    {
                        let ast = Expr::Binary {
                            left: Box::new(Expr::Atom(Atom::Number($a))),
                            operator: $operator,
                            right: Box::new(Expr::Atom(Atom::Number($b))),
                        };
                        let (chunk, bytecode) = generate_bytecode(ast);
                        assert_eq!(
                                bytecode,
                                vec![Opcode::Constant(0), Opcode::Constant(1), Opcode::from($operator)]
                         );
                         assert_eq!(
                                (chunk.read_constant(0), chunk.read_constant(1)),
                                (&Value::Number(a), &Value::Number(b))
                       );
                    }
                )
            *}
        );

        test_binary_with_operators!(a, b, Token::Plus, Token::Minus, Token::Star, Token::Divide);
    }

    #[test]
    fn expr_grouping() {
        let ast = Expr::Grouping {
            expr: Box::new(Expr::Atom(Atom::Bool(true))),
        };
        let (_, bytecode) = generate_bytecode(ast);

        assert_eq!(bytecode, vec![Opcode::True])
    }

    #[test]
    fn expr_unary() {
        let ast = Expr::Unary {
            expr: Box::new(Expr::Atom(Atom::Number(10.0))),
            operator: Token::Minus,
        };
        let (chunk, bytecode) = generate_bytecode(ast);
        assert_eq!(bytecode, vec![Opcode::Constant(0), Opcode::Negate]);
        assert_eq!(chunk.read_constant(0), &Value::Number(10.0));

        let ast = Expr::Unary {
            expr: Box::new(Expr::Binary {
                left: Box::new(Expr::Atom(Atom::Number(10.0))),
                operator: Token::Plus,
                right: Box::new(Expr::Atom(Atom::Number(10.0))),
            }),
            operator: Token::Minus,
        };

        let (chunk, bytecode) = generate_bytecode(ast);
        assert_eq!(
            bytecode,
            vec![
                Opcode::Constant(0),
                Opcode::Constant(1),
                Opcode::Add,
                Opcode::Negate
            ]
        );
        assert_eq!(chunk.read_constant(0), &Value::Number(10.0));
        assert_eq!(chunk.read_constant(1), &Value::Number(10.0));

        let ast = Expr::Unary {
            expr: Box::new(Expr::Atom(Atom::Bool(true))),
            operator: Token::Bang,
        };

        let (_, bytecode) = generate_bytecode(ast);
        assert_eq!(bytecode, vec![Opcode::True, Opcode::Not,]);

        let ast = Expr::Unary {
            expr: Box::new(Expr::Binary {
                left: Box::new(Expr::Atom(Atom::Number(20.0))),
                operator: Token::Plus,
                right: Box::new(Expr::Atom(Atom::Number(10.0))),
            }),
            operator: Token::Bang,
        };

        let (chunk, bytecode) = generate_bytecode(ast);
        assert_eq!(
            bytecode,
            vec![
                Opcode::Constant(0),
                Opcode::Constant(1),
                Opcode::Add,
                Opcode::Not
            ]
        );

        assert_eq!(chunk.read_constant(0), &Value::Number(20.0));
        assert_eq!(chunk.read_constant(1), &Value::Number(10.0));
    }

    const VARIABLE_NAME: &str = "foo";
    const DECLARE_VAR: bool = true;
    const OMIT_VAR: bool = false;

    fn generate_bytecode_with_var<I>(ast: I, should_declare: bool) -> Result<(Chunk, Vec<Opcode>)>
    where
        I: Visitable,
        BytecodeGenerator: Visitor<I>,
    {
        let mut bg = BytecodeGenerator::new();
        if should_declare {
            bg.add_local(VARIABLE_NAME.to_owned());
        }
        let chunk = bg
            .generate(&ast)
            .with_context(|| "Couldn't generate chunk from given ast")?;

        Ok((
            chunk.clone(),
            chunk.into_iter().cloned().collect::<Vec<Opcode>>(),
        ))
    }

    #[test]
    fn expr_var() -> Result<()> {
        // Bytecode generator will handle the Expr::Var if variable has been declared
        // and is stored in the locals vector.

        // Variables that evaluate to value
        let ast = Expr::Var {
            identifier: VARIABLE_NAME.to_owned(),
            is_ref: false,
        };

        let (_, bytecode) = generate_bytecode_with_var(ast, DECLARE_VAR)?;
        assert_eq!(bytecode, vec![Opcode::Var(0)]);

        // Variables that evaluate to reference
        let ast = Expr::Var {
            identifier: VARIABLE_NAME.to_owned(),
            is_ref: true,
        };

        let (_, bytecode) = generate_bytecode_with_var(ast, DECLARE_VAR)?;
        assert_eq!(bytecode, vec![Opcode::VarRef(0)]);

        // Bytecode generator will throw an error if variable referenced by Expr::Var hasn't been declared
        // and isn't stored in the locals vector.

        // Variables that evaluate to value
        let ast = Expr::Var {
            identifier: VARIABLE_NAME.to_owned(),
            is_ref: false,
        };
        assert!(generate_bytecode_with_var(ast, OMIT_VAR).is_err());

        // Variables that evaluate to reference
        let ast = Expr::Var {
            identifier: VARIABLE_NAME.to_owned(),
            is_ref: true,
        };
        assert!(generate_bytecode_with_var(ast, OMIT_VAR).is_err());

        Ok(())
    }

    #[test]
    fn expr_block_no_final_expr() {
        let ast = Expr::Block {
            body: Block {
                body: vec![Stmt::Var {
                    identifier: String::from("foo"),
                    expr: Expr::Atom(Atom::Number(10.0)),
                }],
                final_expr: None,
            },
        };

        let (chunk, bytecode) = generate_bytecode(ast);

        assert_eq!(
            bytecode,
            vec![Opcode::Constant(0), Opcode::Null, Opcode::Block(1)]
        );
        assert_eq!(chunk.read_constant(0), &Value::Number(10.0));
    }

    #[test]
    fn expr_block_final_expr() {
        let ast = Expr::Block {
            body: Block {
                body: vec![],
                final_expr: Some(Box::new(Expr::Atom(Atom::Number(10.0)))),
            },
        };

        let (chunk, bytecode) = generate_bytecode(ast);

        // When no variables are created inside the block, then no Opcode::Block is added
        // because there are no variables to drop
        assert_eq!(bytecode, vec![Opcode::Constant(0)]);
        assert_eq!(chunk.read_constant(0), &Value::Number(10.0));

        let ast = Expr::Block {
            body: Block {
                body: vec![Stmt::Var {
                    identifier: String::from("foo"),
                    expr: Expr::Atom(Atom::Null),
                }],
                final_expr: Some(Box::new(Expr::Atom(Atom::Number(10.0)))),
            },
        };

        let (chunk, bytecode) = generate_bytecode(ast);

        // Opcode::Block is added whenever we declare variables inside the block, so they are dropped
        // at the end of the block.
        assert_eq!(
            bytecode,
            vec![Opcode::Null, Opcode::Constant(0), Opcode::Block(1)]
        );
        assert_eq!(chunk.read_constant(0), &Value::Number(10.0));
    }

    #[test]
    fn if_expr_if() {
        let ast = Expr::If {
            branches: vec![IfBranch {
                branch_type: BranchType::If,
                condition: Expr::Atom(Atom::Bool(true)),
                body: Block {
                    body: vec![Stmt::Var {
                        identifier: String::from("foo"),
                        expr: Expr::Atom(Atom::Bool(true)),
                    }],
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
        let ast = Expr::If {
            branches: vec![
                IfBranch {
                    branch_type: BranchType::If,
                    condition: Expr::Atom(Atom::Bool(false)),
                    body: Block {
                        body: vec![Stmt::Var {
                            identifier: String::from("foo"),
                            expr: Expr::Atom(Atom::Bool(true)),
                        }],
                        final_expr: None,
                    },
                },
                IfBranch {
                    branch_type: BranchType::ElseIf,
                    condition: Expr::Atom(Atom::Bool(true)),
                    body: Block {
                        body: vec![Stmt::Var {
                            identifier: String::from("bar"),
                            expr: Expr::Atom(Atom::Bool(true)),
                        }],
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
        let ast = Expr::If {
            branches: vec![
                IfBranch {
                    branch_type: BranchType::If,
                    condition: Expr::Atom(Atom::Bool(false)),
                    body: Block {
                        body: vec![Stmt::Var {
                            identifier: String::from("foo"),
                            expr: Expr::Atom(Atom::Bool(true)),
                        }],
                        final_expr: None,
                    },
                },
                IfBranch {
                    branch_type: BranchType::Else,
                    // Parser always makes else have a truthful condition
                    condition: Expr::Atom(Atom::Bool(true)),
                    body: Block {
                        body: vec![Stmt::Var {
                            identifier: String::from("bar"),
                            expr: Expr::Atom(Atom::Bool(true)),
                        }],
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
    fn while_expr() {
        let ast = Expr::While {
            condition: Box::new(Expr::Binary {
                left: Box::new(Expr::Atom(Atom::Number(10.0))),
                operator: Token::Less,
                right: Box::new(Expr::Atom(Atom::Number(20.0))),
            }),
            body: Block {
                body: vec![Stmt::Print {
                    expr: Expr::Atom(Atom::Text(String::from("while loop"))),
                }],
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

    // STATEMENTS

    #[test]
    fn stmt_var() {
        let mut bg = BytecodeGenerator::default();

        let ast = Expr::Block {
            body: Block {
                body: vec![Stmt::Var {
                    identifier: String::from(VARIABLE_NAME),
                    expr: Expr::Atom(Atom::Number(10.0)),
                }],
                final_expr: None,
            },
        };

        let chunk = bg
            .generate(&ast)
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

    #[test]
    fn stmt_expr() {
        // Stmt::Expr is just a side effect to kick off the expression stored inside it.
        let ast = Stmt::Expr {
            expr: Expr::Atom(Atom::Number(10.0)),
        };

        let (chunk, bytecode) = generate_bytecode(ast);

        assert_eq!(bytecode, vec![Opcode::Constant(0), Opcode::PopN(1)]);
        assert_eq!(chunk.read_constant(0), &Value::Number(10.0));
    }
}
