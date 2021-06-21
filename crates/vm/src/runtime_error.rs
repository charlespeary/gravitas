pub struct RuntimeError {
    cause: RuntimeErrorCause,
}

pub enum RuntimeErrorCause {
    PoppedFromEmptyStack,
}
