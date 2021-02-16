use std::collections::HashMap;
use std::ops::Neg;

use anyhow::{anyhow, Context, Result};

use crate::constants::{LAMBDA_NAME, MAIN_FUNCTION_NAME};
use crate::{
    bytecode::{Address, Callable, Chunk, Number, Opcode, Value},
    compiler::ProgramOutput,
    std::GLOBALS,
    utils::{logger, logger::LOGGER},
    vm::{
        call_frame::{CallFrame, Callstack, Environments},
        stack::Stack,
        utilities::Utilities,
    },
};

mod call_frame;
pub mod stack;
pub mod utilities;

// Leave current frame to enter frame of e.g called function or stay in the current frame
// to proceed with current execution e.g after calling native function or class
pub enum CallAction {
    Leave,
    Stay,
}

#[derive(Debug, Default)]
pub struct VM<'a> {
    pub utilities: Option<&'a mut Utilities<'a>>,
    // Stack of local values
    stack: Stack,
    // Struct managing environments
    environments: Environments,
    // Stack of function calls
    call_stack: Callstack,
    // Hashmap of global variables, e.g global functions
    globals: HashMap<String, Value>,
    ip: usize,
}

impl<'a> VM<'a> {
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

    pub fn callback(&mut self, callable: Callable) -> Result<()> {
        match &callable {
            Callable::NativeFunction(_) => {
                self.call(callable);
            }
            _ => {
                self.call(callable);
                let frame = self
                    .call_stack
                    .next()
                    .expect("Callback didn't create a new call frame!");
                self.evaluate_frame(frame)?;
            }
        }
        Ok(())
    }

    pub fn call(&mut self, callable: Callable) -> Result<CallAction> {
        let current_env = self.call_stack.current().env_key;
        match callable {
            Callable::Function(function) => {
                LOGGER.log("VM / CALL / FUNCTION", &function.name);
                let env_key = self.environments.create_env(current_env, vec![]);
                self.new_frame(
                    function.chunk.clone(),
                    function.arity,
                    function.name.clone(),
                    env_key,
                );
                self.stack.push(Callable::Function(function).into());
                self.ip = 0;
                Ok(CallAction::Leave)
            }
            Callable::NativeFunction(function) => {
                LOGGER.log("VM / CALL / NATIVE", function.name);
                let args = self.stack.pop_n(function.arity);
                let value = (function.function)(args, self);
                self.stack.push(value);
                // skip the call
                self.ip += 1;
                Ok(CallAction::Stay)
            }
            Callable::Closure(closure) => {
                LOGGER.log("VM / CALL / CLOSURE", &format!("{:?}", closure));
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
                    LAMBDA_NAME.to_owned(),
                    env,
                );

                self.stack
                    .push(Callable::Closure(closure.with_env(env)).into());
                self.ip = 0;
                Ok(CallAction::Leave)
            }
            Callable::Class(class) => {
                LOGGER.log("VM / CALL / CLASS", &class.name);
                // Number of properties provided to this struct initializer
                let to_pop = self.stack.pop_number() as usize;
                let args = self.stack.pop_n(to_pop);
                self.stack.push(class.new_instance(args).into());
                self.ip += 1;
                Ok(CallAction::Stay)
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
        // TODO: not popping frame here might be a bug
        let frame = self
            .call_stack
            .next()
            .expect("Tried to close frame that didn't exist");
        self.environments.decrement_rc(frame.env_key);
        self.ip = frame.return_address;
        self.stack.truncate(frame.stack_start);
    }

    pub fn interpret(&mut self, chunk: Chunk) -> Result<Value> {
        LOGGER.log("VM", "Start of interpretation...");
        // Reset structs values in case if we would like to rerun some code on VM
        self.ip = 0;
        self.environments = Environments::new();
        self.globals = HashMap::default();
        self.call_stack = Callstack::new(CallFrame {
            chunk,
            stack_start: self.stack.len(),
            return_address: self.ip,
            caller_name: MAIN_FUNCTION_NAME.to_owned(),
            env_key: 0,
        });

        let execution_result = self.run();
        LOGGER.log("VM", "End of interpretation...");
        execution_result
    }

    fn assign_at_address(&mut self, address: Address, value: Value) -> Result<()> {
        let frame = self.call_stack.current();

        match address {
            Address::Property(property) => {
                LOGGER.log_dbg("VM / ASSIGN / PROPERTY", &value);
                let mut object = self.get_from_address(*property.top_parent_address.clone())?;

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
                let _ = std::mem::replace(traverse_object, value);

                self.assign_at_address(*property.top_parent_address, object)?;
            }
            Address::Local(address) => {
                LOGGER.log_dbg("VM / ASSIGN / LOCAL", &value);
                self.stack.assign_at(frame.stack_start + address, value);
            }
            Address::Upvalue(index, depth) => {
                LOGGER.log_dbg("VM / ASSIGN / UPVALUE", &value);

                match self.environments.get_value_mut(frame.env_key, index, depth) {
                    Some(old_value) => {
                        let _ = std::mem::replace(old_value, value);
                    }
                    // If it wasn't yet closed then it must live on the stack
                    None => {
                        let address = self.call_stack.lookup(depth).stack_start + index;
                        self.stack.assign_at(address, value);
                    }
                }
            }
            Address::Global(_) => {
                LOGGER.log_dbg("VM / ASSIGN / GLOBAL", &value);

                return Err(anyhow!("Cannot reassign global resource!"));
            }
        };
        self.stack.push(Value::Null);
        Ok(())
    }

    fn get_from_address(&mut self, address: Address) -> Result<Value> {
        let frame = self.call_stack.current();

        Ok(match address {
            Address::Property(property_address) => {
                LOGGER.log_dsp("VM / GET / PROPERTY", &property_address);

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
            Address::Local(index) => {
                LOGGER.log_dsp("VM / GET / LOCAL", index);
                self.stack.get_at_cloned(frame.stack_start + index)
            }
            Address::Upvalue(index, depth) => {
                LOGGER.log("VM / GET / UPVALUE", &format!("{} : {}", index, depth));
                match self.environments.get_value(frame.env_key, index, depth) {
                    Some(value) => value,
                    // If it wasn't yet closed then it must live on the stack
                    None => {
                        let address = self.call_stack.lookup(depth).stack_start + index;
                        self.stack.get_at_cloned(address)
                    }
                }
            }
            Address::Global(name) => {
                LOGGER.log("VM / GET / GLOBAL", &name);
                GLOBALS
                    .get(name.as_str())
                    .cloned()
                    .with_context(|| anyhow!("Global variable {} doesn't exist", name))?
                    .into()
            }
        })
    }

    pub fn evaluate_frame(&mut self, frame: CallFrame) -> Result<()> {
        /// Helper to simplify repetitive usage of binary operators
        macro_rules! bin_op {
            // macro for math operations
            ($operator:tt, 'm') => {{
                let a = self.stack.pop();
                let b = self.stack.pop();
                let result = b $operator a;
                self.stack.push(result?)
            }};
            // macro for logical operations
             ($operator:tt, 'l') => {{
                let a = self.stack.pop();
                let b = self.stack.pop();

                self.stack.push( Value::Bool(b $operator a))
            }};
        }

        while let Some(opcode) = frame.chunk.code.get(self.ip) {
            LOGGER.log("VM", &format!("OPCODE: {:?} IP: {}", opcode, self.ip));
            match opcode {
                Opcode::Constant(index) => {
                    let value = frame.chunk.read_constant(*index).clone();
                    LOGGER.log_dsp("VM / CONSTANT", &value);
                    self.stack.push(value);
                }
                Opcode::True => self.stack.push(Value::Bool(true)),
                Opcode::False => self.stack.push(Value::Bool(false)),
                Opcode::Null => self.stack.push(Value::Null),
                Opcode::Negate => {
                    let value = self.stack.pop();
                    LOGGER.log_dsp("VM / NEGATE", &value);
                    let negated = value.neg()?;
                    self.stack.push(negated);
                }
                Opcode::Not => {
                    let value = self.stack.pop();
                    LOGGER.log_dsp("VM / NOT", &value);
                    let value: bool = value.into();
                    self.stack.push(Value::Bool(value));
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
                    let value = self.stack.pop();
                    let address = self.stack.pop_address();
                    self.assign_at_address(address, value)?;
                }
                Opcode::PopN(amount) => {
                    self.stack.pop_n(*amount);
                }
                Opcode::JumpIfFalse(jump) => {
                    let value: bool = self.stack.pop().into();
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
                    let result = self.stack.pop();
                    self.stack.pop_n(*declared);
                    self.stack.push(result);
                }
                Opcode::CloseValue => {
                    let address = self
                        .stack
                        .pop_address()
                        .into_local()
                        .map_err(|_| anyhow!("ds"))?;
                    let value = self.stack.get_at_cloned(frame.stack_start + address);
                    self.environments.close_value(value, frame.env_key);
                }
                Opcode::CreateClosure => {
                    let current_env = self.call_stack.current().env_key;
                    let closure = self
                        .stack
                        .pop_callable()
                        .into_closure()
                        .map_err(|_| anyhow!("Popped callable that wasn't a closure."))?;

                    self.stack
                        .push(closure.with_enclosing_env(current_env).into());
                }
                Opcode::Break(jump) => {
                    let value = self.stack.pop();
                    self.ip += *jump;
                    self.stack.push(value);
                }
                Opcode::Return => {
                    let result_value = self.stack.pop();
                    self.close_frame();
                    self.stack.push(result_value);
                    return Ok(());
                }
                Opcode::Get => {
                    let address = self.stack.pop_address();
                    let value = self.get_from_address(address)?;
                    self.stack.push(value);
                }
                Opcode::Call => {
                    let caller = self.stack.pop_callable();
                    match self.call(caller)? {
                        CallAction::Stay => {
                            continue;
                        }
                        CallAction::Leave => {
                            return Ok(());
                        }
                    }
                }
            }
            self.ip += 1;
        }

        if frame.caller_name != MAIN_FUNCTION_NAME {
            self.close_frame();
        }

        Ok(())
    }

    pub fn run(&mut self) -> ProgramOutput {
        while let Some(frame) = self.call_stack.next() {
            self.evaluate_frame(frame)?;
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
