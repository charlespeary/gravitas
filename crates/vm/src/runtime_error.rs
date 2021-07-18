#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeError {
    pub cause: RuntimeErrorCause,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RuntimeErrorCause {
    PoppedFromEmptyStack,
    MismatchedTypes,
    StackOverflow,
    ExpectedNumber,
    NotCallable,
}
