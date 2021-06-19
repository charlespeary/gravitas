use call_frame::CallFrame;
use common::{Symbols, SymbolsReader};
use runtime_value::RuntimeValue;
use stack::Stack;

pub(crate) mod call_frame;
pub(crate) mod runtime_value;
pub(crate) mod stack;

struct VM {
    pub(crate) operands: Stack<RuntimeValue>,
    pub(crate) call_stack: Stack<CallFrame>,
    pub(crate) symbols: SymbolsReader,
}

impl VM {
    pub fn new(symbols: SymbolsReader) -> Self {
        Self {
            operands: Stack::new(),
            call_stack: Stack::new(),
            symbols,
        }
    }
}

#[cfg(test)]
mod test {
    use lasso::Rodeo;

    fn new_vm() -> VM {
        let symbols = Rodeo::new().into_reader();
        VM::new(symbols)
    }

    use super::*;
    #[test]
    fn it_works() {
        let mut vm = new_vm();
    }
}
