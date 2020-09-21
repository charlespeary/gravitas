use anyhow::Result;

pub use chunk::Chunk;
pub use opcode::Opcode;
pub use value::{Number, Value};

use crate::parser::{Atom, Expr, Stmt, Token, Visitable, Visitor};

mod chunk;
mod opcode;
mod value;

#[derive(Default)]
pub struct BytecodeGenerator {
    chunk: Chunk,
}

impl BytecodeGenerator {
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
            Expr::Var { identifier } => {
                self.chunk.add_constant(Value::String(identifier.clone()));
                self.chunk.grow(Opcode::GetVar);
            }
            Expr::Block { body } => {
                for stmt in body {
                    stmt.accept(self)?;
                }
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
            Stmt::Expr { expr, terminated } => {
                expr.accept(self)?;
                self.chunk.grow(Opcode::Pop);
            }
            Stmt::Var { expr, identifier } => {
                expr.accept(self)?;
                // self.chunk.add_constant(Value::String(identifier.clone()));
                self.chunk.grow(Opcode::DefineVar);
            }
        }
        // these clones are temporary, since I'm not sure how I will end up generating the bytecode
        Ok(())
    }
}
