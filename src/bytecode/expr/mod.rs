use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, GenerationResult},
    parser::expr::{Expr, Grouping},
};

mod affix;
mod atom;
mod binary;
mod block;
mod call;
mod class;
pub mod closure;
mod conditional;
mod identifier;
mod loops;

impl BytecodeFrom<Box<Expr>> for BytecodeGenerator {
    fn generate(&mut self, expr: &Box<Expr>) -> GenerationResult {
        self.generate(expr.as_ref())?;
        Ok(())
    }
}

impl BytecodeFrom<Expr> for BytecodeGenerator {
    fn generate(&mut self, expr: &Expr) -> GenerationResult {
        match expr {
            Expr::Block(block) => self.generate(block),
            Expr::Identifier(identifier) => self.generate(identifier),
            Expr::Continue(con) => self.generate(con),
            Expr::Break(bre) => self.generate(bre),
            Expr::Grouping(group) => self.generate(group),
            Expr::While(wl) => self.generate(wl),
            Expr::Atom(atom) => self.generate(atom),
            Expr::Affix(affix) => self.generate(affix),
            Expr::If(ifc) => self.generate(ifc),
            Expr::Call(call) => self.generate(call),
            Expr::Return(ret) => self.generate(ret),
            Expr::Binary(binary) => self.generate(binary),
            Expr::Closure(closure) => self.generate(closure),
            Expr::StructInitializer(struct_initializer) => self.generate(struct_initializer),
        }?;
        Ok(())
    }
}

impl BytecodeFrom<Grouping> for BytecodeGenerator {
    fn generate(&mut self, grouping: &Grouping) -> GenerationResult {
        self.generate(&grouping.expr)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::{
        bytecode::{test::generate_bytecode, Opcode},
        parser::expr::Atom,
    };

    use super::*;

    #[test]
    fn expr_grouping() {
        let ast = Grouping {
            expr: Box::new(Expr::Atom(Atom::Bool(true))),
        };
        let (_, bytecode) = generate_bytecode(ast);

        assert_eq!(bytecode, vec![Opcode::True])
    }
}
