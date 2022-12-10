use parser::parse::operator::BinaryOperator;

use crate::Opcode;

impl From<BinaryOperator> for Opcode {
    fn from(data: BinaryOperator) -> Self {
        use BinaryOperator::*;
        // +
        match data {
            Addition => Opcode::Add,
            Subtraction => Opcode::Sub,
            Multiplication => Opcode::Mul,
            Division => Opcode::Div,
            Modulo => Opcode::Mod,
            Power => Opcode::Pow,
            Equals => Opcode::Eq,
            NotEquals => Opcode::Ne,
            LesserThan => Opcode::Lt,
            LesserEquals => Opcode::Le,
            GreaterThan => Opcode::Gt,
            GreaterEquals => Opcode::Ge,
            Or => Opcode::Or,
            And => Opcode::And,
        }
    }
}
