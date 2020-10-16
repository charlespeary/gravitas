use anyhow::Result;

use crate::{bytecode::{Opcode, Value}, BytecodeGenerator};
use crate::parser::{ast::Visitor, expr::Atom};
use crate::bytecode::Bytecode;

impl Visitor<Atom> for BytecodeGenerator {
    type Item = Result<()>;

    fn visit(&mut self, atom: &Atom) -> Self::Item {
        match atom {
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
        }
        Ok(())
    }
}

impl Bytecode for Atom {
    fn emit(&self, emitter: &mut BytecodeGenerator) ->  {
        let opcode = match atom {
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
        }
    }
}




// Create new chunk
// Feed it with vector of statements
// Each statement has lots of expressions
// Each expression deals with some kind of side effects
//