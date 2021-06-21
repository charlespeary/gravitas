#[derive(Clone, Copy, Debug)]
pub struct RuntimeError {
    pub cause: RuntimeErrorCause,
}

#[derive(Clone, Copy, Debug)]
pub enum RuntimeErrorCause {
    PoppedFromEmptyStack,
}
