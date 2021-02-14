use std::collections::HashMap;
use std::ops::Neg;

use anyhow::{anyhow, Context, Result};
use prettytable::{Cell, Row, Table};

use crate::bytecode::stmt::class::ObjectInstance;
use crate::{
    bytecode::{Address, Callable, Chunk, Number, Opcode, Value},
    compiler::ProgramOutput,
    std::GLOBALS,
    utils::logger,
    vm::{
        call_frame::{CallFrame, Environments},
        utilities::Utilities,
    },
};

mod call_frame;
pub mod utilities;

#[derive(Debug, Default)]
pub struct VM<'a> {
    pub utilities: Option<&'a mut Utilities<'a>>,
    // Stack of local values
    stack: Vec<Value>,
    // Struct managing environments
    environments: Environments,
    // Stack of function calls
    call_stack: Vec<CallFrame>,
    // Current call frame
    current_frame: Option<CallFrame>,
    // Hashmap of global variables, e.g global functions
    globals: HashMap<String, Value>,
    ip: usize,
}

impl<'a> VM<'a> {
    fn push_stack(&mut self, value: Value) {
        self.stack.push(value)
    }

    fn truncate_stack(&mut self, to: usize) {
        self.stack.truncate(to);
    }

    fn pop_stack(&mut self) -> Result<Value> {
        let value = self.stack.pop();

        value.with_context(|| "Tried to pop value from an empty stack")
    }

    fn pop_string(&mut self) -> Result<String> {
        self.pop_stack()?
            .into_string()
            .map_err(|_| anyhow!("Accessed value from the stack that wasn't a string."))
    }

    fn pop_object(&mut self) -> Result<ObjectInstance> {
        self.pop_stack()?
            .into_object()
            .map_err(|_| anyhow!("Accessed value from the stack that wasn't an object instance."))
    }

    fn pop_number(&mut self) -> Result<Number> {
        self.pop_stack()?
            .into_number()
            .map_err(|_| anyhow!("Accessed value from the stack that wasn't a number."))
    }

    fn pop_address(&mut self) -> Result<Address> {
        self.pop_stack()?
            .into_address()
            .map_err(|_| anyhow!("Accessed value from the stack that wasn't a reference."))
    }

    fn pop_callable(&mut self) -> Result<Callable> {
        self.pop_stack()?
            .into_callable()
            .map_err(|_| anyhow!("Accessed value from the stack that wasn't callable."))
    }

    fn pop_n(&mut self, n: usize) -> Result<Vec<Value>> {
        let mut values: Vec<Value> = vec![];

        for _ in 0..n {
            values.push(self.pop_stack()?);
        }
        Ok(values)
    }

    pub fn new_frame(&mut self, chunk: Chunk, arity: usize, caller_name: String, env_key: usize) {
        self.call_stack.push(CallFrame {
            chunk,
            // Function should have access to its arguments (arity)
            // in order to allow recursive calls
            stack_start: self.stack.len() - arity,
            // +1 so we skip the opcode that caused call
            return_address: self.ip + 1,
            env_key,
            caller_name,
        });
    }

    fn lookup_call_frame(&self, depth: usize) -> &CallFrame {
        self.call_stack
            .get(self.call_stack.len() - 1 - depth)
            .expect("Tried to lookup call frame above the call stack")
    }

    pub fn callback(&mut self, callable: Callable) -> Result<()> {
        match &callable {
            Callable::NativeFunction(_) => {
                self.call(callable);
            }
            _ => {
                self.call(callable);
                self.evaluate_last_frame();
            }
        }
        Ok(())
    }

    pub fn call(&mut self, callable: Callable) -> Result<()> {
        let current_env = self.lookup_call_frame(0).env_key;
        match callable {
            Callable::Function(function) => {
                let env_key = self.environments.create_env(current_env, vec![]);
                self.new_frame(
                    function.chunk.clone(),
                    function.arity,
                    function.name.clone(),
                    env_key,
                );
                self.stack.push(Callable::Function(function).into());
                self.ip = 0;
                Ok(())
            }
            Callable::NativeFunction(function) => {
                let args = self.pop_n(function.arity)?;
                let value = (function.function)(args, self);
                self.stack.push(value);
                // skip the call
                self.ip += 1;
                Ok(())
            }
            Callable::Closure(closure) => {
                // Create new environment to hold closed values in
                let env = self.environments.create_env(
                    closure
                        .enclosing_env_key
                        .expect("Closure must be enclosed at least by the global env"),
                    closure.referenced_environments.clone(),
                );
                self.new_frame(
                    closure.chunk.clone(),
                    closure.arity,
                    String::from("lambda"),
                    env,
                );

                self.stack
                    .push(Callable::Closure(closure.with_env(env)).into());
                self.ip = 0;
                Ok(())
            }
            Callable::Class(class) => {
                // Number of properties provided to this struct initializer
                let to_pop = self.pop_number()? as usize;
                let args = self.pop_n(to_pop)?;
                self.stack.push(class.new_instance(args).into());
                self.ip += 1;
                Ok(())
            }
        }
    }

    pub fn in_debug(&self) -> bool {
        self.utilities
            .as_ref()
            .map(|u| u.settings.map(|s| s.debug))
            .flatten()
            .unwrap_or(false)
    }

    pub fn close_frame(&mut self) {
        let frame = self
            .call_stack
            .pop()
            .expect("Tried to end call, but there were no call frames available.");
        self.environments.decrement_rc(frame.env_key);
        self.ip = frame.return_address;
        self.truncate_stack(frame.stack_start);
    }

    pub fn interpret(&mut self, chunk: Chunk) -> Result<Value> {
        if self.in_debug() {
            chunk.table();
        }
        // Reset structs values in case if we would like to rerun some code on VM
        self.ip = 0;
        self.environments = Environments::new();
        self.globals = HashMap::default();
        self.call_stack = vec![];

        self.call_stack.push(CallFrame {
            chunk,
            stack_start: self.stack.len(),
            return_address: self.ip,
            caller_name: String::from("main"),
            env_key: 0,
        });
        self.run()
    }

    pub fn evaluate_last_frame(&mut self) -> Result<()> {
        let frame = self
            .call_stack
            .last()
            .cloned()
            .expect("Tried to pop from empty callstack.");
        self.evaluate_frame(frame)
    }

    fn assign_at_address(&mut self, address: Address, value: Value) -> Result<()> {
        let frame = self
            .current_frame
            .as_ref()
            .expect("Tried to access current frame while there was none");

        match address {
            Address::Property(property) => {
                let mut object = self.get_from_address(*property.top_parent_address.clone())?;
                dbg!("BEFORE", &object);

                let mut traverse_object = &mut object;
                for property in property.properties {
                    traverse_object = traverse_object
                        .as_object_mut()
                        .with_context(|| "Tried to set property on a value that isn't an object!")?
                        .properties
                        .get_mut(&property)
                        .with_context(|| {
                            format!(
                                "Tried to set value inside {} which doesn't exist!",
                                property
                            )
                        })?;
                }
                std::mem::replace(traverse_object, value);
                dbg!("AFTER", &object);

                self.assign_at_address(*property.top_parent_address, object)?;
            }
            Address::Local(address) => {
                self.stack[frame.stack_start + address] = value;
            }
            Address::Upvalue(index, depth) => {
                match self.environments.get_value_mut(frame.env_key, index, depth) {
                    Some(old_value) => {
                        let _ = std::mem::replace(old_value, value);
                    }
                    // If it wasn't yet closed then it must live on the stack
                    None => {
                        let address = self.lookup_call_frame(depth).stack_start + index;
                        self.stack[address] = value;
                    }
                }
            }
            Address::Global(_) => {
                return Err(anyhow!("Cannot reassign global resource!"));
            }
        };
        self.push_stack(Value::Null);
        Ok(())
    }

    fn get_from_address(&mut self, address: Address) -> Result<Value> {
        let frame = self
            .current_frame
            .as_ref()
            .expect("Tried to access current frame while there was none");

        Ok(match address {
            Address::Property(property_address) => {
                let mut object = self.get_from_address(*property_address.top_parent_address)?;

                for property in property_address.properties {
                    object = object
                        .into_object()
                        .map_err(|_| anyhow!("This value doesn't contain any properties!"))?
                        .properties
                        .get(&property)
                        .with_context(|| anyhow!("{} doesn't exist", property))?
                        .clone()
                }
                object
            }
            Address::Local(index) => self.stack[frame.stack_start + index].clone(),
            Address::Upvalue(index, depth) => {
                match self.environments.get_value(frame.env_key, index, depth) {
                    Some(value) => value,
                    // If it wasn't yet closed then it must live on the stack
                    None => {
                        let address = self.lookup_call_frame(depth).stack_start + index;
                        self.stack[address].clone()
                    }
                }
            }
            Address::Global(name) => GLOBALS
                .get(name.as_str())
                .cloned()
                .with_context(|| anyhow!("Global variable {} doesn't exist", name))?
                .into(),
        })
    }

    pub fn evaluate_frame(&mut self, frame: CallFrame) -> Result<()> {
        // TODO: Temporary clone
        self.current_frame = Some(frame.clone());
        // if self.in_debug() {
        //     ptable!(
        //         [ cbH4 => "CALL FRAME"],
        //         ["CALLER", "STACK START", "RETURN ADDRESS", "ENVIRONMENT KEY"],
        //         [
        //             &frame.caller_name,
        //             &frame.stack_start,
        //             &frame.return_address,
        //             &frame.env_key
        //         ]
        //     );
        // }
        /// Helper to simplify repetitive usage of binary operators
        macro_rules! bin_op {
            // macro for math operations
            ($operator:tt, 'm') => {{
                let a = self.pop_stack()?;
                let b = self.pop_stack()?;
                let result = b $operator a;
                self.push_stack(result?)
            }};
            // macro for logical operations
             ($operator:tt, 'l') => {{
                let a = self.pop_stack()?;
                let b = self.pop_stack()?;

                self.push_stack( Value::Bool(b $operator a))
            }};
        }

        while let Some(opcode) = frame.chunk.code.get(self.ip) {
            match opcode {
                Opcode::Constant(index) => {
                    self.push_stack(frame.chunk.read_constant(*index).clone())
                }
                Opcode::True => self.push_stack(Value::Bool(true)),
                Opcode::False => self.push_stack(Value::Bool(false)),
                Opcode::Null => self.push_stack(Value::Null),
                Opcode::Negate => {
                    let value = self.pop_stack()?;
                    let negated = value.neg()?;
                    self.push_stack(negated);
                }
                Opcode::Not => {
                    let value: bool = self.pop_stack()?.into();
                    self.push_stack(Value::Bool(value));
                }
                Opcode::Add => bin_op!(+, 'm'),
                Opcode::Subtract => bin_op!(-, 'm'),
                Opcode::Divide => bin_op!(/, 'm'),
                Opcode::Multiply => bin_op!(*, 'm'),
                Opcode::BangEqual => bin_op!(!=, 'l'),
                Opcode::Compare => bin_op!(==, 'l'),
                Opcode::Less => bin_op!(<, 'l'),
                Opcode::LessEqual => bin_op!(<=, 'l'),
                Opcode::Greater => bin_op!(>, 'l'),
                Opcode::GreaterEqual => bin_op!(>=, 'l'),
                Opcode::Assign => {
                    let value = self.pop_stack()?;
                    let address = self.pop_address()?;
                    self.assign_at_address(address, value)?;
                }
                Opcode::PopN(amount) => {
                    self.pop_n(*amount)?;
                }
                Opcode::JumpIfFalse(jump) => {
                    let value: bool = self.pop_stack()?.into();
                    if !value {
                        self.ip += *jump;
                    }
                }
                Opcode::JumpForward(jump) => {
                    self.ip += *jump;
                }
                Opcode::JumpBack(jump) => {
                    self.ip -= *jump;
                    continue;
                }
                Opcode::Block(declared) => {
                    let result = self.pop_stack()?;
                    self.pop_n(*declared)?;
                    self.push_stack(result);
                }
                Opcode::CloseValue => {
                    let address = self
                        .pop_address()?
                        .into_local()
                        .map_err(|_| anyhow!("ds"))?;
                    let value = self.stack[frame.stack_start + address].clone();
                    self.environments.close_value(value, frame.env_key);
                }
                Opcode::CreateClosure => {
                    let current_env = self.lookup_call_frame(0).env_key;
                    let closure = self
                        .pop_callable()?
                        .into_closure()
                        .map_err(|_| anyhow!("Popped callable that wasn't a closure."))?;

                    self.push_stack(closure.with_enclosing_env(current_env).into());
                }
                Opcode::Break(jump) => {
                    let value = self.pop_stack()?;
                    self.ip += *jump;
                    self.push_stack(value);
                }
                Opcode::Return => {
                    let result_value = self.pop_stack()?;
                    self.close_frame();
                    self.push_stack(result_value);
                    return Ok(());
                }
                Opcode::Get => {
                    let address = self.pop_address()?;
                    let value = self.get_from_address(address)?;
                    self.push_stack(value);
                }
                Opcode::Call => {
                    let caller = self.pop_callable()?;
                    self.call(caller)?;
                    return Ok(());
                }
            }
            self.ip += 1;
        }

        self.close_frame();

        Ok(())
    }

    pub fn run(&mut self) -> ProgramOutput {
        while let Some(frame) = self.call_stack.last_mut().cloned() {
            self.evaluate_frame(frame);
        }

        Ok(Value::Null)
    }

    pub fn with_utilities(mut self, utilities: &'a mut Utilities<'a>) -> Self {
        self.utilities = Some(utilities);
        self
    }
}

#[cfg(test)]
mod test {}
