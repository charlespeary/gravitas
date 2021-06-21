use crate::TickResult;

#[derive(Default)]
pub struct Stack<T> {
    pub values: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self { values: vec![] }
    }

    pub fn push(&mut self, item: T) -> usize {
        let index = self.values.len();
        self.values.push(item);
        index
    }

    pub fn pop(&mut self) -> TickResult<T> {
        self.values.pop().ok_or()
    }
}
