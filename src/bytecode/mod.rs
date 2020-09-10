pub use chunk::Chunk;
pub use opcode::{Opcode, Value};

use crate::parser::{Atom, Expr, Visitable, Visitor};

mod chunk;
mod opcode;

#[derive(Default)]
pub struct BytecodeGenerator {
    chunk: Chunk,
}

impl Visitor<Expr> for BytecodeGenerator {
    type Result = ();

    fn visit(&mut self, expr: &Expr) {
        println!("HELLO, {:?}", expr);

        match expr {
            Expr::Atom(atom) => match atom {
                Atom::Number(num) => {
                    self.chunk.add_constant(*num);
                }
                _ => unreachable!(),
            },
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                left.accept(self);
                self.chunk.grow(operator.clone().into());
                right.accept(self);
            }
            Expr::Grouping { expr } => {
                expr.accept(self);
            }
            Expr::Unary { expr } => {
                expr.accept(self);
                self.chunk.grow(Opcode::Negate);
            }
        };
        println!("{:#?}", self.chunk);
    }
}
