use crate::bytecode::stmt::function::Function;

#[derive(Debug)]
pub struct CallFrame<'a> {
    function: &'a Function,
    stack_start: usize,
    return_address: usize,
}
