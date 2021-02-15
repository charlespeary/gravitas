use crate::{
    bytecode::{stmt::class::ObjectInstance, Address, Callable, Number, Value},
    utils::logger::LOGGER,
};

#[derive(Debug, Default)]
pub struct Stack {
    values: Vec<Value>,
}

impl Stack {
    pub fn get_at(&self, index: usize) -> &Value {
        self.values
            .get(index)
            .unwrap_or_else(|| panic!("Stack.get_at failed at index {}", index))
    }

    pub fn get_at_cloned(&self, index: usize) -> Value {
        self.get_at(index).clone()
    }

    pub fn assign_at(&mut self, index: usize, value: Value) {
        self.values.insert(index, value)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn push(&mut self, value: Value) {
        LOGGER.log("STACK / PUSH", &format!("{}", &value));
        self.values.push(value)
    }

    pub fn truncate(&mut self, to: usize) {
        LOGGER.log_dbg("STACK / TRUNCATE", to);
        self.values.truncate(to);
    }

    pub fn pop(&mut self) -> Value {
        let value = self.values.pop();
        LOGGER.log_dbg("STACK / POP", &value);
        value.unwrap_or_else(|| panic!("Stack.pop failed, because it was already empty"))
    }

    pub fn pop_string(&mut self) -> String {
        self.pop()
            .into_string()
            .unwrap_or_else(|_| panic!("Stack.pop_string popped value that wasn't a string"))
    }

    pub fn pop_object(&mut self) -> ObjectInstance {
        self.pop()
            .into_object()
            .unwrap_or_else(|_| panic!("Stack.pop_object popped value that wasn't an object"))
    }

    pub fn pop_number(&mut self) -> Number {
        self.pop()
            .into_number()
            .unwrap_or_else(|_| panic!("Stack.pop_number popped value that wasn't a number"))
    }

    pub fn pop_address(&mut self) -> Address {
        self.pop()
            .into_address()
            .unwrap_or_else(|_| panic!("Stack.pop_address popped value that wasn't an address"))
    }

    pub fn pop_callable(&mut self) -> Callable {
        self.pop()
            .into_callable()
            .unwrap_or_else(|_| panic!("Stack.pop_callable popped value that wasn't callable"))
    }

    pub fn pop_n(&mut self, n: usize) -> Vec<Value> {
        let mut values: Vec<Value> = vec![];

        for _ in 0..n {
            values.push(self.pop());
        }

        values
    }
}
