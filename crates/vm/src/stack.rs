#[derive(Default)]
pub struct Stack<T> {
    pub values: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self { values: vec![] }
    }
}
