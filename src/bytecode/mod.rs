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

    pub fn generate<I>(&mut self, ast: &Vec<I>) -> Result<Chunk>
    where
        I: Visitable,
        Self: Visitor<I>,
    {
        for node in ast {
            node.accept(self);
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

        self.chunk.grow(Opcode::PopN(scope.declared as u8));
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
    type Result = Result<()>;

    fn visit(&mut self, expr: &Expr) -> Self::Result {
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
    type Result = Result<()>;

    fn visit(&mut self, stmt: &Stmt) -> Self::Result {
        match stmt {
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
