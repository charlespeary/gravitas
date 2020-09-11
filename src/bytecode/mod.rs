use anyhow::Result;

pub use chunk::Chunk;
pub use opcode::Opcode;
pub use value::Value;

use crate::parser::{Atom, Expr, Visitable, Visitor};

mod chunk;
mod opcode;
mod value;

#[derive(Default)]
pub struct BytecodeGenerator {
    chunk: Chunk,
}

impl Visitor<Expr> for BytecodeGenerator {
    type Result = Result<Chunk>;

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
                _ => unreachable!(),
            },
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                left.accept(self);
                right.accept(self);
                self.chunk.grow(operator.clone().into());
            }
            Expr::Grouping { expr } => {
                expr.accept(self);
            }
            Expr::Unary { expr } => {
                expr.accept(self);
                self.chunk.grow(Opcode::Negate);
            }
        };
        Ok(self.chunk.clone())
    }
}
