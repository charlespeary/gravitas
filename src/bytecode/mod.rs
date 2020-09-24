use anyhow::{Context, Result};

pub use chunk::Chunk;
pub use opcode::Opcode;
pub use value::{Address, Number, Value};

use crate::parser::{Atom, Expr, Stmt, Token, Visitable, Visitor};

mod chunk;
mod opcode;
mod value;

/// State of the scope / block
#[derive(Default, Debug)]
pub struct Scope {
    /// Amount of declared variables in the given scope.
    pub declared: usize,
}

#[derive(Default)]
pub struct BytecodeGenerator {
    chunk: Chunk,
    locals: Vec<String>,
    scopes: Vec<Scope>,
}

impl BytecodeGenerator {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::default()],
            ..Default::default()
        }
    }

    pub fn generate<I>(&mut self, ast: &[I]) -> Result<Chunk>
    where
        I: Visitable,
        Self: Visitor<I>,
    {
        for node in ast {
            node.accept(self)?;
        }

        // temporary clone until I figure out how to generate bytecode properly
        Ok(self.chunk.clone())
    }

    pub fn begin_scope(&mut self) {
        self.scopes.push(Scope::default())
    }

    pub fn end_scope(&mut self) {
        let scope = self
            .scopes
            .pop()
            .expect("Tried to pop scope that doesn't exist");
        if scope.declared > 0 {
            self.chunk.grow(Opcode::PopN(scope.declared as u8));
        }
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
                    self.chunk.grow((*bool).into());
                }
                Atom::Null => {
                    self.chunk.grow(Opcode::Null);
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
                self.chunk.grow(operator.clone().into());
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
                self.chunk.grow(opcode);
            }
            Expr::Var { identifier, is_ref } => {
                let local = self.find_local(identifier)? as u8;
                let opcode = match *is_ref {
                    true => Opcode::VarRef(local),
                    false => Opcode::Var(local),
                };

                self.chunk.grow(opcode);
            }
            Expr::Block { body } => {
                self.begin_scope();
                for stmt in body {
                    stmt.accept(self)?;
                }
                self.end_scope();
            }
        };
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
                self.chunk.grow(Opcode::Print);
            }
            Stmt::Expr {
                expr,
                terminated: _,
            } => {
                expr.accept(self)?;
            }
            Stmt::Var { expr, identifier } => {
                expr.accept(self)?;
                self.add_local(identifier.clone());
            }
        }
        // these clones are temporary, since I'm not sure how I will end up generating the bytecode
        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
            .generate(&[ast])
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
        println!("{}", text);
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
            .generate(&[ast])
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
    fn expr_block() {
        // Block adds the opcode responsible for dropping the temporary variables at the end of the scope.
        let ast = Expr::Block {
            body: vec![Stmt::Var {
                identifier: String::from("foo"),
                expr: Expr::Atom(Atom::Number(10.0)),
            }],
        };

        let (chunk, bytecode) = generate_bytecode(ast);

        assert_eq!(bytecode, vec![Opcode::Constant(0), Opcode::PopN(1)]);
        assert_eq!(chunk.read_constant(0), &Value::Number(10.0));

        // If there were no variables created in the block, then the PopN opcode is not added.
        let ast = Expr::Block {
            body: vec![Stmt::Expr {
                expr: Expr::Atom(Atom::Number(10.0)),
                terminated: true,
            }],
        };

        let (chunk, bytecode) = generate_bytecode(ast);

        assert_eq!(bytecode, vec![Opcode::Constant(0)]);
        assert_eq!(chunk.read_constant(0), &Value::Number(10.0));
    }

    #[test]
    fn stmt_var() {
        let mut bg = BytecodeGenerator::default();

        let ast = Expr::Block {
            body: vec![Stmt::Var {
                identifier: String::from(VARIABLE_NAME),
                expr: Expr::Atom(Atom::Number(10.0)),
            }],
        };

        let chunk = bg
            .generate(&[ast])
            .expect("Couldn't generate bytecode for given ast");

        let bytecode = into_bytecode(chunk.clone());

        // Bytecode generator adds newly created variable to the locals vector,
        // so it can remember and figure out where variables should be stored on stack.
        assert_eq!(bg.locals, vec![VARIABLE_NAME.to_owned()]);
        // We can search for given local and get back its index on the stack wrapped in a Result.
        // Error is thrown if variable was not created and therefore doesn't exist.
        bg.begin_scope();
        assert_eq!(
            bg.find_local(VARIABLE_NAME)
                .expect("Variable not found in the vector of local variables."),
            0
        );
        bg.end_scope();
        // Variable declaration doesn't add any opcode overhead, because all variables are just temporary values on the stack.
        assert_eq!(bytecode, vec![Opcode::Constant(0), Opcode::PopN(1)]);
        assert_eq!(chunk.read_constant(0), &Value::Number(10.0));
    }

    #[test]
    fn stmt_expr() {
        // Stmt::Expr is just a side effect to kick off the expression stored inside it.
        let ast = Stmt::Expr {
            expr: Expr::Atom(Atom::Number(10.0)),
            terminated: false,
        };

        let (chunk, bytecode) = generate_bytecode(ast);

        assert_eq!(bytecode, vec![Opcode::Constant(0)]);
        assert_eq!(chunk.read_constant(0), &Value::Number(10.0));
    }
}
