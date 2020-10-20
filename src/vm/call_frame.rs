use crate::bytecode::Function;

#[derive(Debug)]
pub struct CallFrame<'a> {
    function: &'a Function,
    stack_start: usize,
    return_address: usize,
}
